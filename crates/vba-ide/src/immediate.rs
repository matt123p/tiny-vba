//! Explicit debugger evaluation boundary. Safe watches never use this path.
use crate::{Host, HostBinding, Ide, Limits, Outcome, Result, ReturnValue, Session, Value, error};
#[derive(Clone, Copy, Debug, Default)]
pub struct EvaluationPolicy {
    pub allow_isolated_calls_and_host_effects: bool,
}
impl Ide {
    /// Execute an explicitly allowed expression in a bounded child VM. Selected
    /// locals are ByVal snapshots; project state is isolated. Host effects are real
    /// and remain subject to the application's ordinary trust/capability checks.
    pub fn evaluate_with_host(
        &self,
        expression: &str,
        host: &mut impl Host,
        policy: EvaluationPolicy,
    ) -> Result<Value> {
        self.handle()?;
        if !policy.allow_isolated_calls_and_host_effects {
            return self.evaluate(expression);
        }
        if self.session_host != Some(host.identity()) {
            return Err(error("evaluation host identity changed"));
        }
        crate::evaluate::expression_ast(expression)?;
        let locals = self.locals();
        if locals.len() > 64
            || locals
                .iter()
                .fold(0usize, |n, (_, _, v)| n.saturating_add(v.payload_bytes()))
                > 1024 * 1024
        {
            return Err(error("Immediate snapshot budget exceeded"));
        }
        let name = (0..256)
            .map(|n| format!("VbaImmediateEvaluation{n}"))
            .find(|name| {
                self.modules.iter().all(|m| {
                    m.procedures
                        .iter()
                        .all(|p| !p.name.eq_ignore_ascii_case(name))
                }) && locals.iter().all(|(local, _, _)| {
                    !local
                        .to_ascii_lowercase()
                        .starts_with(&name.to_ascii_lowercase())
                })
            })
            .ok_or_else(|| error("Immediate entry namespace exhausted"))?;
        let mut prologue = String::new();
        let parameters = locals
            .iter()
            .enumerate()
            .map(|(index, (local, kind, value))| {
                let nominal = self
                    .session
                    .as_ref()
                    .ok_or_else(|| error("no paused session"))?
                    .local_nominal_type(self.handle()?, local)?;
                if let Some(ty) = nominal {
                    let ty = format!("{}.{}", ty.library(), ty.name());
                    if *kind == vba_values::ScalarKind::Variant {
                        let snapshot = format!("{name}Snapshot{index}");
                        prologue.push_str(&format!("Dim {local}() As {ty}\n"));
                        if matches!(value, Value::Array(_)) {
                            prologue.push_str(&format!("{local}={snapshot}\n"));
                        }
                        return Ok(format!("ByVal {snapshot} As Variant"));
                    }
                    return Ok(format!("ByVal {local} As {ty}"));
                }
                Ok(format!("ByVal {local} As Variant"))
            })
            .collect::<Result<Vec<_>>>()?
            .join(", ");
        let wrapper = format!(
            "\nPrivate Function {name}({parameters}) As Variant\n{prologue}{name} = {expression}\nEnd Function\n"
        );
        let active = self
            .active_module()
            .ok_or_else(|| error("no source module"))?
            .id;
        let sources = self
            .modules
            .iter()
            .map(|m| {
                crate::SourceFile::new(
                    m.id,
                    m.name.clone(),
                    if m.id == active {
                        format!("{}{wrapper}", m.text)
                    } else {
                        m.text.clone()
                    },
                )
            })
            .collect::<Result<Vec<_>>>()?;
        let parsed = sources
            .iter()
            .map(|source| vba_syntax::parse(source, vba_syntax::ModuleKind::Standard))
            .collect::<Result<Vec<_>>>()?;
        let program = vba_compiler::compile_modules(
            &parsed,
            &vba_compiler::CompileOptions {
                entry: name,
                revision: format!("immediate-{}", self.revision),
                capabilities: vba_vm::capabilities(),
                imports: self.imports.clone(),
                references: self.reference_bindings.clone(),
                ..Default::default()
            },
        )?;
        let binding =
            if self.imports.is_empty() && self.roots.is_empty() && self.type_resolver.is_none() {
                None
            } else {
                Some(HostBinding::for_program(
                    &program,
                    host.identity(),
                    self.roots.clone(),
                )?)
            };
        let mut child = Session::new_with_type_resolver(
            program,
            1,
            locals.into_iter().map(|(_, _, value)| value).collect(),
            binding,
            Limits {
                instructions: 10_000,
                depth: 32,
                locations: 1024,
                stack_entries: 4096,
                host_calls: 32,
                host_polls: 0,
                payload: 8 * 1024 * 1024,
                callbacks: 0,
                ..Limits::default()
            },
            self.type_resolver.clone(),
        )?;
        loop {
            match child.run_slice(host, 256, &mut |_| Ok(false))? {
                Outcome::Yielded => {}
                Outcome::Completed(ReturnValue::Function(value)) => return Ok(value),
                Outcome::Failed(failure) => {
                    return Err(error(&format!(
                        "Immediate {}{}",
                        failure
                            .number
                            .map_or(String::new(), |n| format!("error {n}: ")),
                        failure.message
                    )));
                }
                _ => {
                    child.cancel(host)?;
                    return Err(error(
                        "Immediate evaluation cannot wait, pause or schedule callbacks",
                    ));
                }
            }
        }
    }
}
