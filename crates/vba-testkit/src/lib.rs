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
