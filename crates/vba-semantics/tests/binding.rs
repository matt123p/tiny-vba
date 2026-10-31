use std::error::Error;
use vba_diagnostics::{SourceFile, SourceId};
use vba_semantics::{Expr, ExprKind, Statement, StatementKind, bind};
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

#[test]
fn with_blocks_bind_a_hidden_object_slot_and_innermost_leading_dots() -> Result<(), Box<dyn Error>>
{
    let p = bind(
        &[parsed(
            "Function Answer() As Long\nDim w As Object\nSet w=Nothing\nWith w\n.Value=1\nWith w\n.Value=2\nEnd With\nEnd With\nEnd Function",
        )?],
        "Answer",
    )?;
    let proc = p.procedures.first().ok_or("procedure")?;
    let slots: Vec<_> = proc
        .locals
        .iter()
        .enumerate()
        .filter(|(_, l)| l.synthetic)
        .map(|(i, _)| i)
        .collect();
    assert_eq!(slots.len(), 2, "one hidden With slot per With block");
    let mut depth = 0usize;
    let mut innermost = None;
    for statement in &proc.body {
        if let StatementKind::Assign {
            slot, object: true, ..
        } = &statement.kind
            && slots.contains(slot)
        {
            depth += 1;
            innermost = Some(*slot);
        }
    }
    assert_eq!(depth, 2);
    // Leading-dot members inside the inner block resolve to the innermost slot.
    let uses: Vec<_> = collect_foreign_targets(&proc.body);
    assert!(uses.contains(&innermost.ok_or("inner slot")?));
    Ok(())
}
fn collect_foreign_targets(body: &[Statement]) -> Vec<usize> {
    let mut out = Vec::new();
    for statement in body {
        match &statement.kind {
            StatementKind::Foreign(expr) | StatementKind::Assign { value: expr, .. } => {
                collect_expr_targets(expr, &mut out);
            }
            StatementKind::If {
                arms, otherwise, ..
            } => {
                for arm in arms {
                    collect_expr_targets(&arm.condition, &mut out);
                    out.extend(collect_foreign_targets(&arm.body));
                }
                out.extend(collect_foreign_targets(otherwise));
            }
            _ => (),
        }
    }
    out
}
fn collect_expr_targets(expr: &Expr, out: &mut Vec<usize>) {
    if let ExprKind::Foreign { target, .. } = &expr.kind
        && let ExprKind::Local(slot) = &target.kind
    {
        out.push(*slot);
    }
}

#[test]
fn leading_dots_outside_a_with_block_are_refused() -> Result<(), Box<dyn Error>> {
    let source = parsed("Sub Demo()\n.Value=1\nEnd Sub")?;
    assert!(bind(&[source], "Demo").is_err());
    Ok(())
}

#[test]
fn project_declarations_shadow_host_globals() -> Result<(), Box<dyn Error>> {
    let import = vba_foreign::Import {
        name: "Helper".into(),
        root: "Workbook".into(),
        member: Some("Helper".into()),
        kind: vba_foreign::InvokeKind::Get,
    };
    let p = vba_semantics::bind_with_imports(
        &[parsed(
            "Function Answer() As Long\nAnswer=Helper\nEnd Function\nFunction Helper() As Long\nHelper=42\nEnd Function",
        )?],
        "Answer",
        &[import],
    )?;
    let proc = p
        .procedures
        .iter()
        .find(|p| p.signature.name == "Answer")
        .ok_or("Answer")?;
    // `Helper` resolves to the project Function call, not the host import.
    assert!(matches!(
        proc.body.first().ok_or("assign")?.kind,
        StatementKind::Assign { .. }
    ));
    assert!(!format!("{:?}", proc.body.first().ok_or("assign")?.kind).contains("Foreign"));
    Ok(())
}

#[test]
fn document_implicit_me_members_precede_host_globals() -> Result<(), Box<dyn Error>> {
    let import = vba_foreign::Import {
        name: "Range".into(),
        root: "Excel".into(),
        member: Some("Range".into()),
        kind: vba_foreign::InvokeKind::Call,
    };
    let fallback = import.clone();
    let source = parse(
        &SourceFile::new(
            SourceId(9),
            "Sheet.cls".into(),
            "Attribute VB_Name = \"SheetCode\"\nSub Changed()\nRange(\"A1\").Value=5\nEnd Sub"
                .into(),
        )?,
        ModuleKind::Document,
    )?;
    let main = parse(
        &SourceFile::new(
            SourceId(10),
            "Main.bas".into(),
            "Sub Main()\nEnd Sub".into(),
        )?,
        ModuleKind::Standard,
    )?;
    let mut members = std::collections::BTreeMap::new();
    members.insert("sheetcode".to_string(), vec!["Range".to_string()]);
    let p = vba_semantics::bind_with_references(
        &[source, main],
        "Main",
        &[import],
        &vba_foreign::ReferenceSet::default(),
        &members,
    )?;
    let proc = p.procedures.first().ok_or("procedure")?;
    let StatementKind::Foreign(expr) = &proc.body.first().ok_or("statement")?.kind else {
        return Err("member statement".into());
    };
    let ExprKind::Foreign { target, member, .. } = &expr.kind else {
        return Err("foreign".into());
    };
    assert_eq!(member, "Value");
    let ExprKind::Foreign { target, member, .. } = &target.kind else {
        return Err("range dispatch".into());
    };
    assert_eq!(member, "Range");
    let ExprKind::Local(slot) = &target.kind else {
        return Err("receiver".into());
    };
    let receiver = proc.locals.get(*slot).ok_or("local")?;
    assert!(receiver.receiver, "bare Range dispatches on Me");
    // Without the declaration the same name falls back to the host import.
    let p = vba_semantics::bind_with_references(
        &[
            parse(
                &SourceFile::new(
                    SourceId(9),
                    "Sheet.cls".into(),
                    "Attribute VB_Name = \"SheetCode\"\nSub Changed()\nRange(\"A1\").Value=5\nEnd Sub".into(),
                )?,
                ModuleKind::Document,
            )?,
            parse(
                &SourceFile::new(SourceId(10), "Main.bas".into(), "Sub Main()\nEnd Sub".into())?,
                ModuleKind::Standard,
            )?,
        ],
        "Main",
        &[fallback],
        &vba_foreign::ReferenceSet::default(),
        &std::collections::BTreeMap::new(),
    )?;
    let proc = p.procedures.first().ok_or("procedure")?;
    let StatementKind::Foreign(expr) = &proc.body.first().ok_or("statement")?.kind else {
        return Err("member statement".into());
    };
    let ExprKind::Foreign { target, .. } = &expr.kind else {
        return Err("foreign".into());
    };
    let ExprKind::Foreign { target, .. } = &target.kind else {
        return Err("range dispatch".into());
    };
    let ExprKind::Local(slot) = &target.kind else {
        return Err("receiver".into());
    };
    let receiver = proc.locals.get(*slot).ok_or("local")?;
    assert!(
        receiver.host_root,
        "without declaration Range is the host global"
    );
    Ok(())
}
