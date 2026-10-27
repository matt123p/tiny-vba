//! Source breakpoints and frame-identity stepping over the VM's headless checkpoint hooks.
#![forbid(unsafe_code)]
/// Version of this native Rust public contract.
pub const API_VERSION: u32 = 1;

use std::collections::BTreeSet;
use vba_diagnostics::{Diagnostic, Span, StatementId};
use vba_vm::{CheckpointView, Outcome, PauseReason, Session, SessionInstance};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Step {
    Into,
    Over,
    Out,
}
struct Target {
    step: Step,
    frame: u64,
    instance: SessionInstance,
    ancestors: Vec<u64>,
}
#[derive(Default)]
pub struct Controller {
    breakpoints: BTreeSet<(SessionInstance, u64, u32)>,
    target: Option<Target>,
}
impl Controller {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn set_breakpoint(
        &mut self,
        session: &Session,
        revision: &str,
        statement: StatementId,
    ) -> Result<Vec<Span>, Diagnostic> {
        if session.program().program().revision != revision {
            return Err(Diagnostic::new(
                "VBA0034",
                "breakpoint source revision mismatch",
            ));
        }
        let spans: Vec<_> = session
            .program()
            .program()
            .procedures
            .iter()
            .flat_map(|p| p.checkpoints.iter())
            .filter(|c| c.visible && c.statement == statement)
            .map(|c| c.span)
            .collect();
        if spans.is_empty() {
            return Err(Diagnostic::new(
                "VBA0034",
                "statement has no visible executable checkpoint",
            ));
        }
        if self.breakpoints.len() >= 4096 {
            return Err(Diagnostic::new("VBA0034", "breakpoint limit"));
        }
        self.breakpoints
            .insert((session.instance(), statement.source.0, statement.ordinal));
        Ok(spans)
    }
    pub fn remove_breakpoint(&mut self, statement: StatementId) {
        self.breakpoints.retain(|(_, source, ordinal)| {
            (*source, *ordinal) != (statement.source.0, statement.ordinal)
        });
    }
    pub fn should_pause(&self, view: &CheckpointView) -> Result<bool, Diagnostic> {
        if !view.visible {
            return Ok(false);
        }
        if self.breakpoints.contains(&(
            view.instance,
            view.statement.source.0,
            view.statement.ordinal,
        )) {
            return Ok(true);
        }
        Ok(self.target.as_ref().is_some_and(|t| {
            t.instance == view.instance
                && match t.step {
                    Step::Into => true,
                    Step::Over => view.frame == t.frame || t.ancestors.contains(&view.frame),
                    Step::Out => t.ancestors.contains(&view.frame),
                }
        }))
    }
    pub fn continue_execution(&mut self, session: &mut Session) -> Result<(), Diagnostic> {
        session.continue_execution()?;
        self.target = None;
        Ok(())
    }
    pub fn step(&mut self, session: &mut Session, step: Step) -> Result<(), Diagnostic> {
        let Outcome::Paused(pause) = session.outcome() else {
            return Err(Diagnostic::new("VBA0034", "step requires paused session"));
        };
        if pause.reason == PauseReason::Fault {
            return Err(Diagnostic::new("VBA0034", "cannot step a pending fault"));
        }
        let frame = pause.handle.frame;
        let instance = pause.handle.instance;
        let ancestors = pause.frames.iter().skip(1).map(|f| f.id).collect();
        session.continue_execution()?;
        self.target = Some(Target {
            step,
            frame,
            instance,
            ancestors,
        });
        Ok(())
    }
    pub fn request_pause(&mut self, session: &mut Session) {
        session.request_pause();
    }
    pub fn break_on_errors(&mut self, session: &mut Session, enabled: bool) {
        session.set_break_on_error(enabled);
    }
}
