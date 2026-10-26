//! Deterministic generic host double. Scripted responses, owned traces, no Office objects or IO.
#![forbid(unsafe_code)]
use std::collections::VecDeque;
use vba_foreign::{ForeignError, Host, HostCall, Reply};
use vba_values::ObjectRef;

#[derive(Clone, Debug, PartialEq)]
pub enum Event {
    Invoke(HostCall),
    Resume(u64),
    Cancel(u64),
}
pub struct ScriptedHost {
    replies: VecDeque<Reply>,
    pub trace: Vec<Event>,
    limit: usize,
    target: ObjectRef,
}
impl ScriptedHost {
    pub fn new(target: ObjectRef, replies: Vec<Reply>) -> Self {
        Self {
            replies: replies.into(),
            trace: Vec::new(),
            limit: 2048,
            target,
        }
    }
    pub fn remaining(&self) -> usize {
        self.replies.len()
    }
    fn next(&mut self, event: Event) -> Reply {
        if self.trace.len() >= self.limit {
            return failure("host trace budget exceeded");
        }
        self.trace.push(event);
        self.replies
            .pop_front()
            .unwrap_or_else(|| failure("host script exhausted"))
    }
}
impl Host for ScriptedHost {
    fn identity(&self) -> u64 {
        self.target.host
    }
    fn invoke(&mut self, call: HostCall) -> Reply {
        if call.target != self.target {
            return Reply::Error(ForeignError {
                number: 91,
                description: "stale or foreign object reference".into(),
            });
        }
        self.next(Event::Invoke(call))
    }
    fn resume(&mut self, token: u64) -> Reply {
        self.next(Event::Resume(token))
    }
    fn cancel(&mut self, token: u64) {
        if self.trace.len() < self.limit {
            self.trace.push(Event::Cancel(token));
        }
    }
}
fn failure(message: &str) -> Reply {
    Reply::Error(ForeignError {
        number: 5,
        description: message.into(),
    })
}

/// Bounded reusable embedding driver: returns at pause/wait/terminal without polling/replay.
pub fn run_to_boundary(session:&mut vba_vm::Session,host:&mut impl Host,controller:&vba_debug::Controller,max_slices:u32)->Result<vba_vm::Outcome,vba_diagnostics::Diagnostic> {
    if max_slices==0 || max_slices>65536 {return Err(vba_diagnostics::Diagnostic::new("VBA0034","invalid test driver allowance"));}
    for _ in 0..max_slices {
        let quota=session.limits().slice.min(256);
        let outcome=session.run_slice(host,quota,&mut |view| controller.should_pause(view))?;
        if outcome!=vba_vm::Outcome::Yielded {return Ok(outcome);}
    }Err(vba_diagnostics::Diagnostic::new("VBA0031","embedding driver slice allowance exceeded"))
}
/// Find a visible source statement by its verbatim text, avoiding hard-coded bytecode PCs.
pub fn statement(program:&vba_ir::VerifiedProgram,text:&str)->Result<vba_diagnostics::StatementId,vba_diagnostics::Diagnostic> {
    for procedure in &program.program().procedures {
        let source=&program.program().sources.get(procedure.source.0 as usize).ok_or_else(|| vba_diagnostics::Diagnostic::new("VBA0034","source missing"))?.file;
        for checkpoint in &procedure.checkpoints {if checkpoint.visible && source.slice(checkpoint.span)?==text {return Ok(checkpoint.statement);}}
    }Err(vba_diagnostics::Diagnostic::new("VBA0034","visible statement not found"))
}
