use std::error::Error;
use vba_diagnostics::{SourceFile, SourceId};
use vba_semantics::{ExprKind, StatementKind, bind};
use vba_syntax::{ModuleKind, parse};
use vba_values::{ScalarKind, Value};
fn parsed(text: &str) -> Result<vba_syntax::ParseResult, Box<dyn Error>> {
    Ok(parse(
        &SourceFile::new(SourceId(8), "binding.bas".into(), text.into())?,
        ModuleKind::Standard,
    )?)
}
#[test]
fn parameters_result_user_and_implicit_locals_have_stable_layout() -> Result<(), Box<dyn Error>> {
    let p = bind(
        &[parsed(
            "Function Answer(ByVal seed As Long) As Long\nDim X As Integer\nx=1\nimplicit=seed\nAnswer=implicit\nEnd Function",
        )?],
        "aNsWeR",
    )?;
    let proc = p.procedures.first().ok_or("procedure")?;
    let layout: Vec<_> = proc
        .locals
        .iter()
        .map(|l| (l.name.as_str(), l.kind, l.parameter, l.result))
        .collect();
    assert_eq!(
        layout,
        [
            ("seed", ScalarKind::Long, true, false),
            ("Answer", ScalarKind::Long, false, true),
            ("X", ScalarKind::Integer, false, false),
            ("implicit", ScalarKind::Variant, false, false)
        ]
    );
    Ok(())
}
#[test]
fn constants_are_typed_and_never_writable_and_explicit_errors_have_exact_span()
-> Result<(), Box<dyn Error>> {
    let p = bind(
        &[parsed(
            "Const Seed=7\nFunction Answer() As Long\nAnswer=seed\nEnd Function",
        )?],
        "Answer",
    )?;
    let statement = p
        .procedures
        .first()
        .and_then(|p| p.body.first())
        .ok_or("assignment")?;
    assert!(
        matches!(&statement.kind,StatementKind::Assign {value,..} if matches!(value.kind,ExprKind::Constant(Value::Integer(7))))
    );
    assert!(
        bind(
            &[parsed("Const Seed=7\nSub Answer()\nseed=8\nEnd Sub")?],
            "Answer"
        )
        .is_err()
    );
    let parsed = parsed("Option Explicit\nSub Answer()\nunknown=1\nEnd Sub")?;
    let error = bind(std::slice::from_ref(&parsed), "Answer")
        .err()
        .ok_or("reject")?;
    let span = error.span.ok_or("span")?;
    assert_eq!(parsed.module.source.slice(span)?, "unknown");
    assert_eq!(parsed.module.source.position(span)?, (3, 1));
    Ok(())
}
#[test]
fn binding_bounds_constant_expansion_slots_and_procedure_counts() -> Result<(), Box<dyn Error>> {
    let text = format!(
        "Const Large=\"{}\"\nFunction Answer() As String\n{}End Function",
        "a".repeat(20_000),
        "Answer=Large\n".repeat(2000)
    );
    let error = bind(&[parsed(&text)?], "Answer")
        .err()
        .ok_or("expansion must be bounded")?;
    assert_eq!(error.code, "VBA0111");
    let mut text = "Sub Answer()\n".to_owned();
    for i in 0..4097 {
        text.push_str(&format!("Dim v{i}\n"));
    }
    text.push_str("End Sub");
    assert_eq!(
        bind(&[parsed(&text)?], "Answer").err().ok_or("slots")?.code,
        "VBA0111"
    );
    let mut text = "Sub Answer()\nEnd Sub\n".to_owned();
    for i in 0..4096 {
        text.push_str(&format!("Sub P{i}()\nEnd Sub\n"));
    }
    assert_eq!(
        bind(&[parsed(&text)?], "Answer")
            .err()
            .ok_or("procedures")?
            .code,
        "VBA0111"
    );
    Ok(())
}
#[test]
fn invalid_syntax_and_byref_launch_are_rejected_before_lowering() -> Result<(), Box<dyn Error>> {
    assert!(bind(&[parsed("Sub Answer()\nx=)\nEnd Sub")?], "Answer").is_err());
    let error = bind(&[parsed("Sub Answer(ByRef x As Long)\nEnd Sub")?], "Answer")
        .err()
        .ok_or("launch")?;
    assert!(error.message.contains("entry parameters"));
    assert!(
        bind(
            &[parsed("Sub Answer()\nDim x As UnknownType\nEnd Sub")?],
            "Answer"
        )
        .is_err()
    );
    Ok(())
}
