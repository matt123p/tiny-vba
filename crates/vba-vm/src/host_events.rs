//! Explicit host-authorized events; no ambient event subscription or COM activation.
use super::*;
use vba_values::{NominalType, ObjectRef};

/// An event parameter declared by the embedding's supported host profile.
#[derive(Clone, Debug)]
pub struct HostEventParameter {
    pub kind: ScalarKind,
    pub by_ref: bool,
    pub nominal: Option<NominalType>,
}

/// One validated live subscription, scoped to the session that discovered it.
#[derive(Clone, Debug)]
pub struct HostEventHandler {
    receiver: ObjectRef,
    procedure: ir::ProcedureId,
}

impl Session {
    /// Snapshot connected sinks in connection order, validating every handler before any runs.
    /// Hosts own event names, signatures, authorization and cross-sink ByRef propagation.
    pub fn host_event_handlers(
        &mut self,
        publisher: ObjectRef,
        event: &str,
        parameters: &[HostEventParameter],
        host: &impl Host,
    ) -> Result<Vec<HostEventHandler>> {
        self.check_host(host)?;
        if self.lifecycle_closed()
            || self.stale
            || !(self.terminal() || matches!(self.state, Outcome::Waiting))
        {
            return Err(command(
                "host events require a current completed invocation",
            ));
        }
        if publisher.host == 0
            || publisher.host != host.identity()
            || event.is_empty()
            || event.len() > 128
            || parameters.len() > 64
            || !event
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b == b'_')
        {
            return Err(command("invalid host event identity/signature"));
        }
        let mut found = Vec::new();
        let mut work = 0u64;
        let program = self.program.program();
        for instance in self
            .instances
            .iter()
            .filter(|instance| instance.alive && !instance.terminating)
        {
            work = work.saturating_add(program.classes.len() as u64);
            let class = program
                .classes
                .iter()
                .find(|class| class.source == instance.source)
                .ok_or_else(|| invariant("host event sink class"))?;
            for (field, location) in class.fields.iter().zip(&instance.fields) {
                work = work.saturating_add(1);
                if work
                    > self
                        .limits
                        .instructions
                        .saturating_sub(self.counters.instance_work)
                {
                    return Err(resource());
                }
                if !field.with_events || self.store.read(*location)? != &Value::Object(publisher) {
                    continue;
                }
                let name = format!("{}_{}", field.name, event);
                work = work.saturating_add(class.methods.len() as u64);
                if work
                    > self
                        .limits
                        .instructions
                        .saturating_sub(self.counters.instance_work)
                {
                    return Err(resource());
                }
                let Some(method) = class
                    .methods
                    .iter()
                    .find(|method| method.name.eq_ignore_ascii_case(&name))
                else {
                    continue;
                };
                let procedure = program
                    .procedures
                    .get(method.procedure.0 as usize)
                    .ok_or_else(|| invariant("host event handler"))?;
                if method.kind != ir::ForeignKind::Call
                    || procedure.kind != ir::ProcedureKind::Sub
                    || procedure.parameters.len() != parameters.len()
                {
                    return Err(command("host event handler signature mismatch"));
                }
                for (index, (actual, expected)) in
                    procedure.parameters.iter().zip(parameters).enumerate()
                {
                    work = work.saturating_add(program.nominal_locals.len() as u64);
                    if work
                        > self
                            .limits
                            .instructions
                            .saturating_sub(self.counters.instance_work)
                    {
                        return Err(resource());
                    }
                    let nominal = program
                        .nominal_locals
                        .iter()
                        .find(|local| {
                            local.procedure == method.procedure && local.slot.0 as usize == index
                        })
                        .map(|local| &local.ty);
                    if !actual.required
                        || actual.array.is_some()
                        || actual.kind != expected.kind
                        || (actual.mode == ir::ParameterMode::ByRef) != expected.by_ref
                        || nominal != expected.nominal.as_ref()
                    {
                        return Err(command("host event handler parameter mismatch"));
                    }
                }
                if found.len() >= self.limits.callbacks.min(256) {
                    return Err(resource());
                }
                found.try_reserve(1).map_err(|_| resource())?;
                let order = *self
                    .event_connections
                    .get(location)
                    .ok_or_else(|| invariant("host event connection"))?;
                found.push((
                    order,
                    HostEventHandler {
                        receiver: instance.object,
                        procedure: method.procedure,
                    },
                ));
            }
        }
        self.counters.instance_work = self
            .counters
            .instance_work
            .checked_add(work)
            .ok_or_else(resource)?;
        found.sort_by_key(|(order, _)| *order);
        Ok(found.into_iter().map(|(_, handler)| handler).collect())
    }

    /// Deliver one member of a previously validated subscription snapshot exactly once.
    /// Ordinary callback outputs carry ByRef values back to the host after completion.
    pub fn schedule_host_event_handler(
        &mut self,
        handler: &HostEventHandler,
        arguments: Vec<Value>,
    ) -> Result<usize> {
        if self.lifecycle_closed()
            || self.stale
            || !(self.terminal() || matches!(self.state, Outcome::Waiting))
        {
            return Err(command(
                "host event handler requires a current completed invocation",
            ));
        }
        let source = self.instance_object(handler.receiver)?.source;
        if self
            .program
            .program()
            .procedures
            .get(handler.procedure.0 as usize)
            .is_none_or(|procedure| procedure.source != source)
        {
            return Err(command("host event handler belongs to another session"));
        }
        self.enqueue_callback(handler.procedure, arguments, Some(handler.receiver))
    }
}
