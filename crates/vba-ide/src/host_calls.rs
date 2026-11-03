//! Explicit, bounded calls on an existing waiting project session.
use super::*;
use std::collections::VecDeque;
pub use vba_vm::NestedCallResult;
#[derive(Clone, Debug)]
pub enum SuspendedCall {
    HostEvent {
        handler: HostEventHandler,
        arguments: Vec<Value>,
    },
    Procedure {
        name: String,
        arguments: Vec<Value>,
    },
    DocumentEvent {
        code_name: String,
        handler: String,
        arguments: Vec<Value>,
        owner: Option<ObjectRef>,
    },
}
pub(crate) struct PendingCall {
    token: u64,
    depth: usize,
    remaining: usize,
}
pub(crate) type Results = VecDeque<(u64, NestedCallResult)>;
impl Ide {
    /// Apply an embedding's fatal nested-error policy, preserving normal host cancellation
    /// and form token ownership. Wrong tokens cannot abort a different invocation.
    pub fn abort_host_wait(
        &mut self,
        token: u64,
        failure: vba_foreign::ForeignError,
        host: &mut impl Host,
    ) -> Result<()> {
        if self.stale || self.session_host != Some(host.identity()) {
            return Err(error("fatal host completion identity mismatch"));
        }
        let session = self
            .session
            .as_mut()
            .ok_or_else(|| error("no project session"))?;
        let result = if self.forms.is_empty() {
            session.abort_host_wait(token, failure, host)
        } else {
            let mut gate = std::mem::take(&mut self.form_gate);
            let result = session.abort_host_wait(
                token,
                failure,
                &mut form_project::FormHost {
                    host,
                    gate: &mut gate,
                },
            );
            self.form_gate = gate;
            result
        };
        result?;
        self.suspended_host_calls.clear();
        self.host_call_results.clear();
        if let Outcome::Failed(f) = session.outcome() {
            let mut diagnostic = Diagnostic::new(
                "VBA_EXECUTION",
                format!("Error {}: {}", f.number.unwrap_or(5), f.message),
            );
            if let Some(span) = f.frames.first().and_then(|f| f.span) {
                diagnostic = diagnostic.at(span);
            }
            self.diagnostics = vec![diagnostic.clone()];
            self.navigate_diagnostic(&diagnostic);
        }
        Ok(())
    }
    /// The host must already be waiting on this exact token. Execution still advances through
    /// normal debugger ticks, with the same VM, persistent state and cumulative limits.
    pub fn start_suspended_call(
        &mut self,
        token: u64,
        call: SuspendedCall,
        host: &mut impl Host,
    ) -> Result<()> {
        if self.stale
            || self.session_host != Some(host.identity())
            || self.mode() != Mode::Waiting
            || self.suspended_host_calls.len() >= 8
            || self.host_call_results.len() >= self.limits.callbacks
        {
            return Err(error("host call identity/lifecycle/budget mismatch"));
        }
        self.suspended_host_calls
            .try_reserve(1)
            .map_err(|_| error("host call allocation"))?;
        self.host_call_results
            .try_reserve(1)
            .map_err(|_| error("host call result allocation"))?;
        let session = self
            .session
            .as_mut()
            .ok_or_else(|| error("no project session"))?;
        if session.pending_host_token() != Some(token) || session.pending_callbacks() != 0 {
            return Err(error(
                "host call requires its own wait and an empty callback queue",
            ));
        }
        if let SuspendedCall::Procedure { name, .. } = &call {
            let (module, name) = name
                .rsplit_once('.')
                .map_or((None, name.as_str()), |(m, n)| (Some(m), n));
            let program = session.program().program();
            let mut candidates = program.procedures.iter().filter(|p| {
                p.name.eq_ignore_ascii_case(name)
                    && !program.classes.iter().any(|c| c.source == p.source)
                    && module.is_none_or(|m| {
                        program
                            .sources
                            .get(p.source.0 as usize)
                            .is_some_and(|s| s.module.eq_ignore_ascii_case(m))
                    })
            });
            let p = candidates
                .next()
                .ok_or_else(|| error("unknown public project procedure"))?;
            if candidates.next().is_some()
                || !self
                    .modules
                    .iter()
                    .find(|m| m.id == p.declaration.source())
                    .is_some_and(|m| {
                        m.procedures
                            .iter()
                            .any(|p| p.public && p.name.eq_ignore_ascii_case(name))
                    })
            {
                return Err(error("ambiguous or private project procedure"));
            }
        }
        session.clear_callback_outputs();
        let scheduled = match call {
            SuspendedCall::HostEvent { handler, arguments } => {
                session.schedule_host_event_handler(&handler, arguments)
            }
            SuspendedCall::Procedure { name, arguments } => {
                session.schedule_host_procedure(&name, arguments)
            }
            SuspendedCall::DocumentEvent {
                code_name,
                handler,
                arguments,
                owner,
            } => session
                .schedule_document_callback_with_owner(&code_name, &handler, arguments, owner),
        };
        let count = match scheduled {
            Ok(count) => count,
            Err(e) => {
                session.discard_queued_host_callbacks();
                return Err(e);
            }
        };
        if let Err(e) = session.start_suspended_callback(host) {
            session.discard_queued_host_callbacks();
            return Err(e);
        }
        self.suspended_host_calls.push(PendingCall {
            token,
            depth: session.suspended_depth(),
            remaining: count,
        });
        Ok(())
    }
    pub fn pending_host_token(&self) -> Option<u64> {
        self.session.as_ref().and_then(Session::pending_host_token)
    }
    pub fn take_host_call_result(&mut self) -> Option<(u64, NestedCallResult)> {
        self.host_call_results.pop_front()
    }
    pub(crate) fn finish_host_call(&mut self, host: &mut impl Host) -> Result<bool> {
        let Some(call) = self.suspended_host_calls.last_mut() else {
            return Ok(false);
        };
        let session = self
            .session
            .as_mut()
            .ok_or_else(|| error("no project session"))?;
        if session.suspended_depth() != call.depth
            || !matches!(
                session.outcome(),
                Outcome::Completed(_) | Outcome::Failed(_) | Outcome::Cancelled
            )
        {
            return Ok(false);
        }
        let Some(result) = session.finish_host_callback(host)? else {
            self.suspended_host_calls.clear();
            self.host_call_results.clear();
            return Ok(true);
        };
        call.remaining = call.remaining.saturating_sub(1);
        if result.is_err() || call.remaining == 0 {
            let token = call.token;
            self.suspended_host_calls.pop();
            self.host_call_results.push_back((token, result));
            session.discard_queued_host_callbacks();
        } else {
            session.start_suspended_callback(host)?;
        }
        Ok(true)
    }
}
