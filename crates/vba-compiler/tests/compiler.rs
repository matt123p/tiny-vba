use std::error::Error;
use vba_compiler::{CompileOptions, compile, compile_modules};
use vba_diagnostics::{SourceFile, SourceId};
use vba_ir::{self as ir, Instruction as I};
use vba_values::{ScalarKind, Value};
fn source(text: &str) -> Result<SourceFile, Box<dyn Error>> {
    Ok(SourceFile::new(
        SourceId(17),
        "test.bas".into(),
        text.into(),
    )?)
}
fn build(text: &str) -> Result<ir::VerifiedProgram, Box<dyn Error>> {
    Ok(compile(&source(text)?, &CompileOptions::default())?)
}
#[test]
fn answer_golden_is_verified_and_deterministic() -> Result<(), Box<dyn Error>> {
    let file = SourceFile::new(
        SourceId(0),
        "Arithmetic.bas".into(),
        include_str!("../../../examples/pure-vba/Arithmetic.bas").into(),
    )?;
    let a = compile(&file, &CompileOptions::default())?;
    let b = compile(&file, &CompileOptions::default())?;
    let output = ir::disassemble(&a)?;
    assert_eq!(output, ir::disassemble(&b)?);
    assert_eq!(
        output,
        include_str!("../../../tests/semantics/m2/answer.disasm")
    );
    let proc = a.program().procedures.first().ok_or("procedure")?;
    assert_eq!(proc.max_stack, 3);
    assert_eq!(proc.locals.first().ok_or("result")?.kind, ScalarKind::Long);
    assert_eq!(
        a.program().constants,
        [Value::Integer(6), Value::Integer(7)]
    );
    assert!(matches!(
        proc.code.get(4),
        Some(I::Binary(ir::BinaryOp::Multiply))
    ));
    Ok(())
}
#[test]
fn nested_calls_and_colon_statements_map_to_their_own_checkpoint() -> Result<(), Box<dyn Error>> {
    let p = build(
        "Option Explicit\nFunction Answer() As Long\nAnswer = F(1) + F(F(2)): Answer = Answer + 3\nEnd Function\nFunction F(ByVal n As Long) As Long\nF = n\nEnd Function\n",
    )?;
    let proc = p.program().procedures.first().ok_or("procedure")?;
    assert_eq!(proc.calls.len(), 3);
    assert!(proc.calls.iter().all(|c| c.checkpoint == ir::SiteId(0)));
    assert_ne!(
        proc.checkpoints.first().ok_or("first")?.statement,
        proc.checkpoints.get(1).ok_or("second")?.statement
    );
    assert_eq!(
        p.program().constants,
        [Value::Integer(1), Value::Integer(2), Value::Integer(3)]
    );
    assert_eq!(proc.max_stack, 3); // destination Location, first result, nested argument
    Ok(())
}
#[test]
fn if_for_byref_and_hidden_scratch_lower_to_verified_blocks() -> Result<(), Box<dyn Error>> {
    let file = source(include_str!("../../../examples/pure-vba/M2Control.bas"))?;
    let p = compile(&file, &CompileOptions::default())?;
    let proc = p.program().procedures.first().ok_or("procedure")?;
    assert_eq!(p.program().procedures.len(), 3);
    assert_eq!(
        proc.locals
            .iter()
            .filter(|l| l.role == ir::SlotRole::Scratch && !l.visible)
            .count(),
        3
    );
    assert!(p.program().required_features.contains("byref-locals"));
    let loop_sites: Vec<_> = proc
        .checkpoints
        .iter()
        .filter(|c| matches!(c.role, ir::SiteRole::LoopInit | ir::SiteRole::LoopTest))
        .collect();
    assert_eq!(loop_sites.len(), 3);
    assert!(loop_sites.iter().all(|c| {
        c.statement
            == loop_sites
                .first()
                .map(|c| c.statement)
                .unwrap_or(c.statement)
    }));
    assert_eq!(loop_sites.iter().filter(|c| !c.visible).count(), 1);
    for instruction in &proc.code {
        if let I::Jump(target) = instruction {
            assert!(matches!(
                proc.code.get(target.0 as usize),
                Some(I::Checkpoint(_))
            ));
        }
    }
    Ok(())
}
#[test]
fn unreachable_code_is_removed_but_source_statements_remain() -> Result<(), Box<dyn Error>> {
    let p = build("Function Answer() As Long\nAnswer=1\nExit Function\nAnswer=99\nEnd Function\n")?;
    let proc = p.program().procedures.first().ok_or("procedure")?;
    assert_eq!(proc.statements.len(), 4);
    assert_eq!(proc.checkpoints.len(), 3);
    assert_eq!(p.program().constants, [Value::Integer(1)]);
    Ok(())
}
#[test]
fn unsupported_and_invalid_binding_never_produces_verified_program() -> Result<(), Box<dyn Error>> {
    for (text, needle) in [
        (
            "Option Explicit\nFunction Answer() As Long\nx=1\nEnd Function",
            "undeclared",
        ),
        (
            "Function Answer() As Long\nDim x As Long, X As Long\nEnd Function",
            "duplicate",
        ),
        (
            "Function Answer() As Long\nAnswer=MissingProcedure(1)\nEnd Function",
            "unknown procedure",
        ),
        (
            "Function Answer() As Long\nAnswer=F()\nEnd Function\nSub F()\nEnd Sub",
            "Sub used as value",
        ),
        (
            "Function Answer() As Long\nCall F()\nEnd Function\nSub F(ByVal x As Long)\nEnd Sub",
            "argument count",
        ),
        (
            "Function Answer() As Long\nDim x As Integer\nF x\nEnd Function\nSub F(ByRef x As Long)\nEnd Sub",
            "ByRef declared type",
        ),
        (
            "Function Answer() As Long\nF (1)\nEnd Function\nSub F(ByRef x As Long)\nEnd Sub",
            "parenthesized",
        ),
        (
            "Function Answer() As Long\nAnswer=F(x:=1)\nEnd Function\nFunction F(ByVal x As Long) As Long\nEnd Function",
            "named/omitted",
        ),
        (
            "Function Answer() As Long\nEnd Function\nSub F(Optional x As Long=1)\nEnd Sub",
            "optional",
        ),
        (
            "Function Answer() As Long\nDim i As Long\nFor i=1 To 2\nNext j\nEnd Function",
            "Next counter",
        ),
        (
            "Function Answer() As Long\nExit Sub\nEnd Function",
            "Exit outside",
        ),
        (
            "Function Answer() As Long\nExit For\nEnd Function",
            "Exit outside",
        ),
        (
            "Dim x As Long\nFunction Answer() As Long\nEnd Function",
            "persistent module",
        ),
        (
            "Function Answer() As Long\nAnswer=&H10\nEnd Function",
            "numeric literal",
        ),
        (
            "Function Answer() As Long\nDim i As Long\nFor i=2 To 1 Step -1\nNext i\nEnd Function",
            "constant expression",
        ),
    ] {
        let result = compile(&source(text)?, &CompileOptions::default());
        let Err(error) = result else {
            return Err(format!("accepted {text}").into());
        };
        assert!(error.message.contains(needle), "{needle}: {error}");
        assert!(error.span.is_some());
    }
    Ok(())
}
#[test]
fn cross_module_visibility_and_local_shadowing_are_bound() -> Result<(), Box<dyn Error>> {
    let first=SourceFile::new(SourceId(1),"a.bas".into(),"Option Explicit\nConst Seed=6\nFunction Answer() As Long\nDim seed As Long\nseed=7\nAnswer=F(seed)\nEnd Function".into())?;
    let second = SourceFile::new(
        SourceId(2),
        "b.bas".into(),
        "Public Function F(ByVal n As Long) As Long\nF=n\nEnd Function".into(),
    )?;
    let parsed = [
        vba_syntax::parse(&first, vba_syntax::ModuleKind::Standard)?,
        vba_syntax::parse(&second, vba_syntax::ModuleKind::Standard)?,
    ];
    let program = compile_modules(&parsed, &CompileOptions::default())?;
    assert_eq!(program.program().sources.len(), 2);
    let mut private = parsed.clone();
    private
        .get_mut(1)
        .and_then(|p| p.module.procedures.first_mut())
        .ok_or("procedure")?
        .visibility = vba_syntax::Visibility::Private;
    assert!(compile_modules(&private, &CompileOptions::default()).is_err());
    let assignment = SourceFile::new(
        SourceId(1),
        "a.bas".into(),
        "Function Answer() As Long\nF=1\nEnd Function".into(),
    )?;
    let mut invalid = parsed.clone();
    *invalid.first_mut().ok_or("module")? =
        vba_syntax::parse(&assignment, vba_syntax::ModuleKind::Standard)?;
    assert!(compile_modules(&invalid, &CompileOptions::default()).is_err());
    Ok(())
}
#[test]
fn custom_targets_and_verifier_budgets_are_honored() -> Result<(), Box<dyn Error>> {
    let file = source("Function Answer() As Long\nAnswer=6*7\nEnd Function")?;
    let mut options = CompileOptions::default();
    options.capabilities.binary.remove(&ir::BinaryOp::Multiply);
    assert_eq!(
        compile(&file, &options).err().ok_or("must reject")?.code,
        "VBA0022"
    );
    options = CompileOptions::default();
    options.limits.work = 1;
    assert_eq!(
        compile(&file, &options).err().ok_or("must reject")?.code,
        "VBA0021"
    );
    Ok(())
}

#[test]
fn first_emitted_constant_order_preserves_exact_tags_and_deduplicates() -> Result<(), Box<dyn Error>>
{
    let p = build("Function Answer() As Long\nAnswer=7: Answer=7&: Answer=7\nEnd Function")?;
    assert_eq!(p.program().constants, [Value::Integer(7), Value::Long(7)]);
    let q = build("Function Answer() As String\nAnswer=\"hello\": Answer=\"hello\"\nEnd Function")?;
    assert_eq!(q.program().constants.len(), 1);
    Ok(())
}
