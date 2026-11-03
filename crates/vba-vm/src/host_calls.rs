//! Explicit host-mediated calls into the current project. No Office dispatch or IO.
use super::*;
pub type NestedCallResult = std::result::Result<foreign::Completion, foreign::ForeignError>;
impl Session {
    /// A host may choose a fatal policy for a returned nested error. This stops all parked
    /// invocations at the exact original wait; it never injects a catchable caller error.
    pub fn abort_host_wait(
        &mut self,
        token: u64,
        error: foreign::ForeignError,
        host: &mut impl Host,
    ) -> Result<()> {
        self.check_host(host)?;
        if !matches!(self.outcome(), Outcome::Waiting)
            || self.pending_host_token() != Some(token)
            || error.number == 0
            || error.description.len() > self.limits.payload
        {
            return Err(command("invalid fatal host wait completion"));
        }
        let failure = self.failure(FailureKind::Runtime, Some(error.number), error.description)?;
        self.cancel(host)?;
        self.state = Outcome::Failed(failure);
        Ok(())
    }
    /// Resolve a standard-module procedure, including optional arguments, without executing it.
    pub fn schedule_host_procedure(&mut self, name: &str, arguments: Vec<Value>) -> Result<usize> {
        if name.is_empty() || name.len() > 256 || arguments.len() > 256 {
            return Err(command("invalid host procedure call"));
        }
        let (module, name) = name
            .rsplit_once('.')
            .map_or((None, name), |(m, n)| (Some(m), n));
        let mut matches = self
            .program
            .program()
            .procedures
            .iter()
            .enumerate()
            .filter(|(_, p)| {
                p.name.eq_ignore_ascii_case(name)
                    && !self
                        .program
                        .program()
                        .classes
                        .iter()
                        .any(|c| c.source == p.source)
                    && module.is_none_or(|m| {
                        self.program
                            .program()
                            .sources
                            .get(p.source.0 as usize)
                            .is_some_and(|s| s.module.eq_ignore_ascii_case(m))
                    })
            });
        let (index, procedure) = matches
            .next()
            .ok_or_else(|| command("unknown host procedure"))?;
        if matches.next().is_some() {
            return Err(command("ambiguous host procedure"));
        }
        if arguments.len() > procedure.parameters.len()
            && !procedure.parameters.last().is_some_and(|p| p.param_array)
        {
            return Err(runtime_diagnostic(runtime::Error::Vba {
                number: 450,
                message: "wrong argument count".into(),
            }));
        }
        let mut prepared = Vec::new();
        reserve(&mut prepared, procedure.parameters.len())?;
        for (i, parameter) in procedure.parameters.iter().enumerate() {
            if parameter.param_array {
                let tail = arguments.get(i..).unwrap_or_default();
                let mut array = if tail.is_empty() {
                    vba_values::VbaArray::empty(vba_values::ScalarKind::Variant, 0)
                } else {
                    vba_values::VbaArray::new(
                        vba_values::ScalarKind::Variant,
                        vec![(0, i32::try_from(tail.len() - 1).map_err(|_| resource())?)],
                        false,
                    )?
                };
                for (index, value) in tail.iter().enumerate() {
                    array.set(index, value.clone())?;
                }
                prepared.push(Value::Array(array));
                continue;
            }
            let v = arguments.get(i).cloned().unwrap_or(Value::Missing);
            if matches!(v, Value::Missing) {
                if parameter.required {
                    return Err(runtime_diagnostic(runtime::Error::Vba {
                        number: 449,
                        message: "argument required".into(),
                    }));
                }
                prepared.push(parameter.default.clone().unwrap_or(Value::Missing));
            } else {
                prepared.push(v);
            }
        }
        self.enqueue_callback(
            ir::ProcedureId(u32::try_from(index).map_err(|_| resource())?),
            prepared,
            None,
        )
    }
    pub fn discard_queued_host_callbacks(&mut self) {
        self.callbacks.clear();
    }
    pub fn suspended_depth(&self) -> usize {
        self.suspended_invocations.len()
    }
    /// Restore a parked caller after success or a catchable VBA failure. Resource/protocol
    /// failures retain the existing fail-closed cancellation path and cannot replenish budgets.
    pub fn finish_host_callback(
        &mut self,
        host: &mut impl Host,
    ) -> Result<Option<NestedCallResult>> {
        self.check_host(host)?;
        let result = match &self.state {
            Outcome::Completed(value) => Ok(foreign::Completion {
                value: match value {
                    ReturnValue::Sub => Value::Empty,
                    ReturnValue::Function(v) => v.clone(),
                },
                writes: self.take_callback_outputs().unwrap_or_default(),
            }),
            Outcome::Failed(f) if f.kind == FailureKind::Runtime => Err(foreign::ForeignError {
                number: f.number.unwrap_or(5),
                description: f.message.clone(),
            }),
            Outcome::Failed(_) | Outcome::Cancelled => {
                self.finish_suspended_callback(host)?;
                return Ok(None);
            }
            _ => return Err(command("host callback is not terminal")),
        };
        self.restore_suspended()?;
        Ok(Some(result))
    }
}
