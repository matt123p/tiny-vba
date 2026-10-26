//! Owned, host-neutral M0 invocation/suspension prototype. No Office catalogue or ambient IO.
#![forbid(unsafe_code)]
use vba_diagnostics::Diagnostic;
use vba_values::{Location, Locations, ObjectRef, Value};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum InvokeKind {
    Call,
    Get,
    Let,
    Set,
}
#[derive(Clone, Debug, PartialEq)]
pub enum ArgumentValue {
    Omitted,
    ByVal(Value),
    ByRef(Location),
}
#[derive(Clone, Debug, PartialEq)]
pub struct Argument {
    pub name: Option<String>,
    pub value: ArgumentValue,
}
#[derive(Clone, Debug, PartialEq)]
pub struct Invocation {
    pub kind: InvokeKind,
    pub target: ObjectRef,
    pub member: String,
    pub arguments: Vec<Argument>,
}
/// Host receives snapshots; it never retains a VM/document borrow or a writable location.
#[derive(Clone, Debug, PartialEq)]
pub struct HostArgument {
    pub name: Option<String>,
    pub value: Value,
    pub by_ref: bool,
}
#[derive(Clone, Debug, PartialEq)]
pub struct HostCall {
    pub kind: InvokeKind,
    pub target: ObjectRef,
    pub member: String,
    pub arguments: Vec<HostArgument>,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ForeignError {
    pub number: i32,
    pub description: String,
}
#[derive(Clone, Debug, PartialEq)]
pub struct Completion {
    pub value: Value,
    /// Argument indexes, in write order. Aliased locations observe the last write.
    pub writes: Vec<(usize, Value)>,
}
#[derive(Clone, Debug, PartialEq)]
pub enum Reply {
    Complete(Completion),
    Error(ForeignError),
    Wait(u64),
}
pub trait Host {
    /// Stable identity, unique per host instance during the embedding session.
    fn identity(&self) -> u64;
    fn invoke(&mut self, call: HostCall) -> Reply;
    fn resume(&mut self, token: u64) -> Reply;
    fn cancel(&mut self, token: u64);
}
#[derive(Clone, Debug, PartialEq)]
pub enum Outcome {
    Complete(Value),
    Error(ForeignError),
    Suspended,
}
struct Pending {
    host: u64,
    waits: usize,
    token: u64,
    locations: Vec<Option<Location>>,
}
/// One outstanding call per session. Caller must cancel before close/disconnect.
/// The host must preserve successful side effects across errors/cancellation; no macro rollback.
#[derive(Default)]
pub struct Session {
    pending: Option<Pending>,
}
impl Session {
    pub fn is_suspended(&self) -> bool {
        self.pending.is_some()
    }
    pub fn invoke(
        &mut self,
        host: &mut impl Host,
        invocation: Invocation,
        store: &mut Locations,
    ) -> Result<Outcome, Diagnostic> {
        self.invoke_checked(host, invocation, store, &mut |_, _| Ok(()))
    }
    pub fn invoke_checked(
        &mut self,
        host: &mut impl Host,
        invocation: Invocation,
        store: &mut Locations,
        validate: &mut impl FnMut(Option<Location>, &Value) -> Result<(), Diagnostic>,
    ) -> Result<Outcome, Diagnostic> {
        if self.pending.is_some() {
            return Err(protocol("call already suspended"));
        }
        if invocation.target.host != host.identity() {
            return Err(protocol("target belongs to another host"));
        }
        if invocation.member.is_empty()
            || invocation.member.len() > 256
            || invocation.arguments.len() > 256
        {
            return Err(protocol("invalid member or argument budget"));
        }
        let mut arguments = Vec::new();
        let mut locations = Vec::new();
        let mut names: Vec<String> = Vec::new();
        for arg in invocation.arguments {
            if let Some(name) = &arg.name {
                if name.is_empty()
                    || name.len() > 256
                    || names.iter().any(|n| n.eq_ignore_ascii_case(name))
                {
                    return Err(protocol("invalid or duplicate argument name"));
                }
                names.push(name.clone());
            } else if !names.is_empty() {
                return Err(protocol("positional argument follows named argument"));
            }
            let (value, location) = match arg.value {
                ArgumentValue::Omitted => (Value::Missing, None),
                ArgumentValue::ByVal(value) => (value, None),
                ArgumentValue::ByRef(location) => (store.read(location)?.clone(), Some(location)),
            };
            value.validate()?;
            arguments.push(HostArgument {
                name: arg.name,
                value,
                by_ref: location.is_some(),
            });
            locations.push(location);
        }
        let reply = host.invoke(HostCall {
            kind: invocation.kind,
            target: invocation.target,
            member: invocation.member,
            arguments,
        });
        self.accept(reply, locations, store, host.identity(), 0, validate)
    }
    pub fn resume(
        &mut self,
        host: &mut impl Host,
        store: &mut Locations,
    ) -> Result<Outcome, Diagnostic> {
        self.resume_checked(host, store, &mut |_, _| Ok(()))
    }
    pub fn resume_checked(
        &mut self,
        host: &mut impl Host,
        store: &mut Locations,
        validate: &mut impl FnMut(Option<Location>, &Value) -> Result<(), Diagnostic>,
    ) -> Result<Outcome, Diagnostic> {
        self.check_host(host)?;
        let pending = self
            .pending
            .take()
            .ok_or_else(|| protocol("no suspended call"))?;
        // Validate lifetimes before asking the host to perform more work.
        for location in pending.locations.iter().flatten() {
            if let Err(error) = store.read(*location) {
                host.cancel(pending.token);
                return Err(error);
            }
        }
        if pending.waits >= 1024 {
            host.cancel(pending.token);
            return Err(protocol("suspension budget exceeded"));
        }
        let reply = host.resume(pending.token);
        self.accept(
            reply,
            pending.locations,
            store,
            pending.host,
            pending.waits,
            validate,
        )
    }
    fn check_host(&self, host: &impl Host) -> Result<(), Diagnostic> {
        if self
            .pending
            .as_ref()
            .is_some_and(|p| p.host != host.identity())
        {
            return Err(protocol("pending call belongs to another host"));
        }
        Ok(())
    }
    pub fn cancel(&mut self, host: &mut impl Host) -> Result<bool, Diagnostic> {
        self.check_host(host)?;
        if let Some(pending) = self.pending.take() {
            host.cancel(pending.token);
            Ok(true)
        } else {
            Ok(false)
        }
    }
    fn accept(
        &mut self,
        reply: Reply,
        locations: Vec<Option<Location>>,
        store: &mut Locations,
        host: u64,
        waits: usize,
        validate: &mut impl FnMut(Option<Location>, &Value) -> Result<(), Diagnostic>,
    ) -> Result<Outcome, Diagnostic> {
        match reply {
            Reply::Wait(token) => {
                self.pending = Some(Pending {
                    host,
                    waits: waits + 1,
                    token,
                    locations,
                });
                Ok(Outcome::Suspended)
            }
            Reply::Error(error) => {
                if error.number == 0 || error.description.len() > 65536 {
                    return Err(protocol("host error budget exceeded"));
                }
                Ok(Outcome::Error(error))
            }
            Reply::Complete(completion) => {
                completion.value.validate()?;
                validate(None, &completion.value)?;
                if completion.writes.len() > 256 {
                    return Err(protocol("write budget exceeded"));
                }
                // Validate the entire patch before any VM write. Host effects are already committed.
                for (index, value) in &completion.writes {
                    let location = locations
                        .get(*index)
                        .copied()
                        .flatten()
                        .ok_or_else(|| protocol("write to non-ByRef argument"))?;
                    store.read(location)?;
                    value.validate()?;
                    validate(Some(location), value)?;
                }
                for (index, value) in completion.writes {
                    let location = locations
                        .get(index)
                        .copied()
                        .flatten()
                        .ok_or_else(|| protocol("invalid write index"))?;
                    store.write(location, value)?;
                }
                Ok(Outcome::Complete(completion.value))
            }
        }
    }
}
fn protocol(message: &str) -> Diagnostic {
    Diagnostic::new("VBA0006", message)
}
