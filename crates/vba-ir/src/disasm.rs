use crate::*;
use std::fmt::{self, Write};
use vba_diagnostics::Diagnostic;
struct Output(String);
impl Write for Output {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        if self
            .0
            .len()
            .checked_add(s.len())
            .is_none_or(|n| n > 64 * 1024 * 1024)
        {
            return Err(fmt::Error);
        }
        self.0.try_reserve(s.len()).map_err(|_| fmt::Error)?;
        self.0.push_str(s);
        Ok(())
    }
}
/// Stable diagnostic text for verified programs. No binary bytecode format is implied.
pub fn disassemble(verified: &VerifiedProgram) -> Result<String, Diagnostic> {
    let mut out = Output(String::new());
    render(&mut out, verified.program())
        .map_err(|_| Diagnostic::new("VBA0021", "disassembly output budget exceeded"))?;
    Ok(out.0)
}
fn scalar(out: &mut Output, v: &Value) -> fmt::Result {
    match v {
        Value::Single(v) => write!(out, "Single(bits=0x{:08x})", v.to_bits()),
        Value::Double(v) => write!(out, "Double(bits=0x{:016x})", v.to_bits()),
        Value::Date(v) => write!(out, "Date(serial={v},bits=0x{:016x})", v.to_bits()),
        Value::Currency(v) => write!(out, "Currency(scaled={v})"),
        Value::String(v) => {
            write!(out, "String(\"")?;
            for unit in v.units() {
                write!(out, "\\u{unit:04x}")?;
            }
            write!(out, "\")")
        }
        _ => write!(out, "{v:?}"),
    }
}
fn render(out: &mut Output, p: &Program) -> fmt::Result {
    writeln!(
        out,
        "IR {} profile={:?} revision={:?} entry={} launch={:?}",
        p.ir_version, p.profile, p.revision, p.entry.0, p.launch.arguments
    )?;
    writeln!(out, "features={:?}", p.required_features)?;
    for (i, s) in p.sources.iter().enumerate() {
        writeln!(
            out,
            "source {i} id={} module={:?} file={:?}",
            s.file.id().0,
            s.module,
            s.file.name()
        )?;
    }
    for (i, v) in p.constants.iter().enumerate() {
        write!(out, "constant {i} ")?;
        scalar(out, v)?;
        writeln!(out)?;
    }
    for (i, proc) in p.procedures.iter().enumerate() {
        writeln!(
            out,
            "procedure {i} {:?} {:?} source={} declaration={}..{} end={} entry={} result={:?} max_stack={}",
            proc.kind,
            proc.name,
            proc.source.0,
            proc.declaration.start(),
            proc.declaration.end(),
            proc.end_statement.ordinal,
            proc.entry.0,
            proc.result,
            proc.max_stack
        )?;
        for (i, param) in proc.parameters.iter().enumerate() {
            writeln!(
                out,
                "  parameter {i} {:?} {:?} {:?} required={}",
                param.name, param.kind, param.mode, param.required
            )?;
        }
        for (i, param) in proc.parameters.iter().enumerate() {
            if let Some(element) = param.array {
                writeln!(out, "  parameter-array {i} element={element:?}")?;
            }
        }
        for (i, local) in proc.locals.iter().enumerate() {
            writeln!(
                out,
                "  slot {i} {:?} {:?} {:?} visible={} span={:?}",
                local.name, local.kind, local.role, local.visible, local.span
            )?;
        }
        for (i, local) in proc.locals.iter().enumerate() {
            if let Some(element) = local.array {
                writeln!(out, "  slot-array {i} element={element:?}")?;
            }
        }
        for statement in &proc.statements {
            writeln!(
                out,
                "  statement {}:{} span={}..{}",
                statement.id.source.0,
                statement.id.ordinal,
                statement.span.start(),
                statement.span.end()
            )?;
        }
        for (i, site) in proc.checkpoints.iter().enumerate() {
            writeln!(
                out,
                "  site {i} statement={}:{} role={:?} visible={} span={}..{} enter={} resume_next={:?}",
                site.statement.source.0,
                site.statement.ordinal,
                site.role,
                site.visible,
                site.span.start(),
                site.span.end(),
                site.enter_pc.0,
                site.resume_next
            )?;
        }
        for (i, call) in proc.calls.iter().enumerate() {
            writeln!(
                out,
                "  call {i} checkpoint={} callee={} arguments={:?}",
                call.checkpoint.0, call.callee.0, call.arguments
            )?;
        }
        for (i, site) in proc.foreign.iter().enumerate() {
            writeln!(
                out,
                "  foreign {i} checkpoint={} {:?} {:?} arguments={:?}",
                site.checkpoint.0, site.kind, site.member, site.arguments
            )?;
        }
        for (pc, instruction) in proc.code.iter().enumerate() {
            writeln!(out, "  {pc:04} {instruction:?}")?;
        }
    }
    Ok(())
}
