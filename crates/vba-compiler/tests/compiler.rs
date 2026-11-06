use std::error::Error;
use vba_compiler::{CompileOptions, compile, compile_modules};
use vba_diagnostics::{SourceFile, SourceId};
use vba_ir::{self as ir, Instruction as I};
use vba_values::{Intrinsic, IntrinsicReturn, ScalarKind, Value};
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
fn named_intrinsic_arguments_lower_to_ir8_source_order_plans() -> Result<(), Box<dyn Error>> {
    let options = CompileOptions {
        capabilities: ir::Capabilities::ir8_structural(),
        ..Default::default()
    };
    let program = compile(
        &source(
            "Function Answer() As Double\nAnswer=Round(numdecimalplaces:=1,expression:=2.25)\nEnd Function",
        )?,
        &options,
    )?;
    assert_eq!(program.program().ir_version, ir::IR8_VERSION);
    assert!(
        program
            .program()
            .required_features
            .contains("intrinsics-m6a-v1")
    );
    let call = program
        .program()
        .procedures
        .iter()
        .flat_map(|procedure| &procedure.code)
        .find_map(|instruction| match instruction {
            I::IntrinsicCall {
                function,
                supplied_arguments,
                bindings,
                compare_mode,
                ..
            } if *function == vba_values::Intrinsic::Round => {
                Some((*supplied_arguments, bindings, *compare_mode))
            }
            _ => None,
        })
        .ok_or("IR 8 Round call")?;
    assert_eq!(call.0, 2);
    assert_eq!(call.2, vba_values::IntrinsicComparisonMode::Binary);
    assert_eq!(
        call.1,
        &[
            vba_values::IntrinsicArgumentBinding::Evaluated(1),
            vba_values::IntrinsicArgumentBinding::Evaluated(0),
        ]
    );
    assert!(
        compile(
            &source("Function Answer() As Double\nAnswer=Round(expression:=2.25)\nEnd Function")?,
            &CompileOptions::default()
        )
        .is_err()
    );
    Ok(())
}

#[test]
fn intrinsic_binding_rejects_invalid_names_order_and_required_omission()
-> Result<(), Box<dyn Error>> {
    let options = CompileOptions {
        capabilities: ir::Capabilities::ir8_structural(),
        ..Default::default()
    };
    for expression in [
        "Round(unknown:=1)",
        "Round(expression:=1,expression:=2)",
        "Round(expression:=1,2)",
        "Round(numdecimalplaces:=1)",
    ] {
        let source_text = format!("Function Answer() As Double\nAnswer={expression}\nEnd Function");
        assert!(
            compile(&source(&source_text)?, &options).is_err(),
            "{expression}"
        );
    }
    Ok(())
}

#[test]
fn strcomp_is_always_lowered_to_ir8_with_variant_result_and_context_mode()
-> Result<(), Box<dyn Error>> {
    let options = CompileOptions {
        capabilities: ir::Capabilities::ir8_structural(),
        ..Default::default()
    };
    let program = compile(
        &source("Function Answer() As Variant\nAnswer=StrComp(\"a\",\"b\")\nEnd Function")?,
        &options,
    )?;
    assert_eq!(program.program().ir_version, ir::IR8_VERSION);
    let call = program
        .program()
        .procedures
        .iter()
        .flat_map(|procedure| &procedure.code)
        .find_map(|instruction| match instruction {
            I::IntrinsicCall {
                function,
                return_variant,
                compare_mode,
                supplied_arguments,
                bindings,
            } if *function == Intrinsic::StrComp => Some((
                *return_variant,
                *compare_mode,
                *supplied_arguments,
                bindings,
            )),
            _ => None,
        })
        .ok_or("IR 8 StrComp call")?;
    assert_eq!(call.0, IntrinsicReturn::Variant);
    assert_eq!(call.1, vba_values::IntrinsicComparisonMode::Binary);
    assert_eq!(call.2, 2);
    assert_eq!(
        call.3,
        &[
            vba_values::IntrinsicArgumentBinding::Evaluated(0),
            vba_values::IntrinsicArgumentBinding::Evaluated(1),
            vba_values::IntrinsicArgumentBinding::Omitted,
        ]
    );
    Ok(())
}

#[test]
fn strcomp_rejects_invalid_argument_shapes() -> Result<(), Box<dyn Error>> {
    let options = CompileOptions {
        capabilities: ir::Capabilities::ir8_structural(),
        ..Default::default()
    };
    for text in [
        "Function Answer() As Variant\nAnswer=StrComp(\"a\")\nEnd Function",
        "Function Answer() As Variant\nAnswer=StrComp(,\"b\")\nEnd Function",
        "Function Answer() As Variant\nAnswer=StrComp(\"a\",\"b\",0,1)\nEnd Function",
        "Function Answer() As Variant\nAnswer=StrComp(string1:=\"a\",string1:=\"b\",string2:=\"c\")\nEnd Function",
        "Function Answer() As Variant\nAnswer=StrComp(string1:=\"a\",mystery:=\"b\")\nEnd Function",
        "Function Answer() As Variant\nAnswer=StrComp(string1:=\"a\",\"b\")\nEnd Function",
    ] {
        assert!(compile(&source(text)?, &options).is_err(), "{text}");
    }
    Ok(())
}

#[test]
fn local_strcomp_name_shadows_the_intrinsic() -> Result<(), Box<dyn Error>> {
    let options = CompileOptions {
        capabilities: ir::Capabilities::ir8_structural(),
        ..Default::default()
    };
    let program = compile(
        &source(
            "Function Answer() As Variant\nDim StrComp As Object\nAnswer=StrComp(\"a\",\"b\")\nEnd Function",
        )?,
        &options,
    )?;
    let mut instructions = program
        .program()
        .procedures
        .iter()
        .flat_map(|procedure| &procedure.code);
    assert!(
        instructions
            .clone()
            .any(|instruction| matches!(instruction, I::Invoke(_)))
    );
    assert!(!instructions.any(|instruction| matches!(
        instruction,
        I::IntrinsicCall {
            function: Intrinsic::StrComp,
            ..
        }
    )));
    Ok(())
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
            "missing required",
        ),
        (
            "Function Answer() As Long\nDim x As Integer\nF x\nEnd Function\nSub F(ByRef x As Long)\nEnd Sub",
            "ByRef declared type",
        ),
        (
            "Function Answer() As Long\nF unknown:=1\nEnd Function\nSub F(ByRef x As Long)\nEnd Sub",
            "unknown named",
        ),
        (
            "Function Answer() As Long\nAnswer=F(unknown:=1)\nEnd Function\nFunction F(ByVal x As Long) As Long\nEnd Function",
            "unknown named",
        ),
        (
            "Function Answer() As Long\nEnd Function\nSub F(Optional x As Long)\nEnd Sub",
            "typed Optional",
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
            "Dim x As Unknown\nFunction Answer() As Long\nEnd Function",
            "declared type",
        ),
        (
            "Function Answer() As Long\nAnswer=&H10\nEnd Function",
            "numeric literal",
        ),
        (
            "Function Answer() As Long\nDim i As Long\nFor i=2 To 1 Step -1\nNext i\nEnd Function",
            "nonpositive/nonintegral",
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

#[test]
fn imports_lower_to_ir4_hidden_roots_and_refuse_writes() -> Result<(), Box<dyn Error>> {
    let imports = vec![vba_foreign::Import {
        name: "sheet".into(),
        root: "Sheet1".into(),
        member: Some("FillCells".into()),
        kind: vba_foreign::InvokeKind::Call,
    }];
    let options = CompileOptions {
        imports,
        ..Default::default()
    };
    let p = compile(
        &source("Function Answer() As Long\nsheet.FillCells 1\nAnswer=7\nEnd Function")?,
        &options,
    )?;
    let program = p.program();
    assert_eq!(program.ir_version, ir::IR4_VERSION);
    assert!(program.required_features.contains("host-roots-v4"));
    let roots: Vec<_> = program
        .procedures
        .iter()
        .flat_map(|p| p.locals.iter())
        .filter(|l| l.role == ir::SlotRole::HostRoot)
        .collect();
    let [root] = roots.as_slice() else {
        return Err("expected exactly one hidden root".into());
    };
    assert_eq!(root.name, "Sheet1");
    assert!(!root.visible);
    assert!(root.span.is_none());
    assert_eq!(root.kind, ScalarKind::Object);
    // A second import on the same root shares one hidden slot; another root gets its own.
    let options = CompileOptions {
        imports: vec![
            vba_foreign::Import {
                name: "sheet".into(),
                root: "Sheet1".into(),
                member: Some("FillCells".into()),
                kind: vba_foreign::InvokeKind::Call,
            },
            vba_foreign::Import {
                name: "book".into(),
                root: "Book1".into(),
                member: None,
                kind: vba_foreign::InvokeKind::Get,
            },
        ],
        ..Default::default()
    };
    let q = compile(
        &source(
            "Function Answer() As Long\nsheet.FillCells 1\nDim w As Long\nw = book\nsheet.FillCells w\nAnswer=w\nEnd Function",
        )?,
        &options,
    )?;
    let roots: Vec<_> = q
        .program()
        .procedures
        .iter()
        .flat_map(|p| p.locals.iter())
        .filter(|l| l.role == ir::SlotRole::HostRoot)
        .collect();
    assert_eq!(roots.len(), 2);
    // Member writes through an import are refused; the root itself is read-only.
    let options = CompileOptions {
        imports: vec![vba_foreign::Import {
            name: "sheet".into(),
            root: "Sheet1".into(),
            member: Some("FillCells".into()),
            kind: vba_foreign::InvokeKind::Call,
        }],
        ..Default::default()
    };
    let refused = compile(
        &source("Function Answer() As Long\nsheet.FillCells = 4\nAnswer=7\nEnd Function")?,
        &options,
    );
    assert!(refused.is_err());
    // Import names shadow nothing: a declared local takes precedence and stays writable.
    let shadowed = compile(
        &source(
            "Function Answer() As Long\nDim sheet As Long\nsheet = 4\nAnswer=sheet\nEnd Function",
        )?,
        &CompileOptions::default(),
    )?;
    assert_eq!(shadowed.program().ir_version, 1);
    Ok(())
}

#[test]
fn host_optional_defaults_require_ir8_capability_and_metadata_feature() -> Result<(), Box<dyn Error>>
{
    let source = source(
        "Sub Answer()\nCall Work\nEnd Sub\nPublic Sub Work(Optional ByVal n As Long = 7)\nEnd Sub\n",
    )?;
    let modern = compile(
        &source,
        &CompileOptions {
            capabilities: ir::Capabilities::ir8_structural(),
            ..Default::default()
        },
    )?;
    assert!(
        modern
            .program()
            .required_features
            .contains("host-procedure-defaults-v1")
    );
    let work = modern
        .program()
        .procedures
        .iter()
        .find(|p| p.name == "Work")
        .ok_or("procedure")?;
    assert_eq!(
        work.parameters.first().ok_or("parameter")?.default,
        Some(Value::Long(7))
    );
    assert!(!work.parameters.first().ok_or("parameter")?.required);
    let mut malformed = modern.program().clone();
    malformed
        .required_features
        .remove("host-procedure-defaults-v1");
    assert!(
        ir::verify(
            malformed,
            &ir::Capabilities::ir8_structural(),
            &ir::VerifyLimits::default()
        )
        .is_err()
    );
    assert!(
        ir::verify(
            modern.program().clone(),
            &ir::Capabilities::ir7_structural(),
            &ir::VerifyLimits::default()
        )
        .is_err()
    );
    let old = compile(
        &source,
        &CompileOptions {
            capabilities: ir::Capabilities::ir7_structural(),
            ..Default::default()
        },
    )?;
    assert!(
        !old.program()
            .required_features
            .contains("host-procedure-defaults-v1")
    );
    assert!(
        old.program()
            .procedures
            .iter()
            .find(|p| p.name == "Work")
            .ok_or("procedure")?
            .parameters
            .first()
            .ok_or("parameter")?
            .required
    );
    Ok(())
}

#[test]
fn nominal_event_signatures_share_the_bounded_verification_index() -> Result<(), Box<dyn Error>> {
    use vba_syntax::{ModuleKind, parse};
    let parameters = "ByVal a As Publisher,ByVal b As Publisher,ByVal c As Publisher,ByVal d As Publisher,ByVal e As Publisher";
    let publisher = format!("Public Event Changed({parameters})\n");
    let mut sink = String::new();
    for n in 0..32 {
        sink.push_str(&format!("Private WithEvents source{n} As Publisher\nPrivate Sub source{n}_Changed({parameters})\nEnd Sub\n"));
    }
    let mut main = String::from("Sub Main()\n");
    for n in 0..1 {
        main.push_str(&format!("Dim item{n} As Publisher\n"));
    }
    main.push_str("End Sub\n");
    let mut modules = Vec::new();
    for (n, (name, text, kind)) in [
        ("Main.bas", main, ModuleKind::Standard),
        ("Publisher.cls", publisher, ModuleKind::Class),
        ("Sink.cls", sink, ModuleKind::Class),
    ]
    .into_iter()
    .enumerate()
    {
        modules.push(parse(
            &SourceFile::new(SourceId(n as u64 + 1), name.into(), text)?,
            kind,
        )?);
    }
    let options = CompileOptions {
        entry: "Main".into(),
        capabilities: ir::Capabilities::ir11_structural(),
        ..Default::default()
    };
    let program = compile_modules(&modules, &options)?;
    let mut expanded = program.program().clone();
    let main = expanded
        .procedures
        .iter()
        .position(|p| p.name == "Main")
        .ok_or("Main")?;
    let ty = expanded.nominal_locals.first().ok_or("nominal")?.ty.clone();
    while expanded.nominal_locals.len() < 1000 {
        let slot = expanded.procedures.get(main).ok_or("Main")?.locals.len();
        expanded
            .procedures
            .get_mut(main)
            .ok_or("Main")?
            .locals
            .push(ir::Local {
                auto_new: None,
                array: None,
                name: format!("unusedNominal{slot}"),
                kind: ScalarKind::Object,
                role: ir::SlotRole::User,
                span: None,
                visible: false,
            });
        expanded.nominal_locals.push(ir::NominalLocal {
            procedure: ir::ProcedureId(main as u32),
            slot: ir::LocalSlot(slot as u32),
            ty: ty.clone(),
        });
    }
    let program = ir::verify(expanded, &options.capabilities, &options.limits)?;
    let mut invalid = program.program().clone();
    invalid
        .nominal_locals
        .push(invalid.nominal_locals.first().ok_or("nominal")?.clone());
    assert!(ir::verify(invalid, &options.capabilities, &options.limits).is_err());
    Ok(())
}

#[test]
fn event_signatures_keep_nominal_arguments_without_unused_class_field_aliases()
-> Result<(), Box<dyn Error>> {
    use vba_syntax::{ModuleKind, parse};
    let mut publisher = String::new();
    for n in 0..256 {
        publisher.push_str(&format!("Private child{n} As Publisher\n"));
    }
    for n in 0..20 {
        publisher.push_str(&format!(
            "Public Event Changed{n}(ByVal value As Publisher)\n"
        ));
    }
    publisher.push_str("Public Sub Fire(ByVal value As Publisher)\nSet child0=value\nRaiseEvent Changed0(value)\nEnd Sub\n");
    let modules = [
        parse(
            &SourceFile::new(
                SourceId(1),
                "Main.bas".into(),
                "Sub Main()\nDim p As Publisher\nSet p=New Publisher\np.Fire p\nEnd Sub".into(),
            )?,
            ModuleKind::Standard,
        )?,
        parse(
            &SourceFile::new(SourceId(2), "Publisher.cls".into(), publisher)?,
            ModuleKind::Class,
        )?,
    ];
    let compiled = compile_modules(
        &modules,
        &CompileOptions {
            entry: "Main".into(),
            capabilities: ir::Capabilities::ir11_structural(),
            ..Default::default()
        },
    )?;
    let program = compiled.program();
    let class = program
        .classes
        .iter()
        .find(|c| c.name == "Publisher")
        .ok_or("publisher")?;
    assert_eq!(class.fields.len(), 256);
    assert_eq!(class.events.len(), 20);
    assert!(class.fields.iter().all(|field| {
        field
            .nominal
            .as_ref()
            .is_some_and(|ty| ty.name().eq_ignore_ascii_case("Publisher"))
    }));
    for event in &class.events {
        let proc = program.procedures.get(event.0 as usize).ok_or("event")?;
        assert_eq!(proc.parameters.len(), 1);
        assert!(
            proc.locals
                .iter()
                .all(|local| !matches!(local.role, ir::SlotRole::InstanceField(_)))
        );
        assert!(
            program
                .nominal_locals
                .iter()
                .any(|local| local.procedure == *event
                    && local.slot == ir::LocalSlot(0)
                    && local.ty.name().eq_ignore_ascii_case("Publisher"))
        );
    }
    let fire = program
        .procedures
        .iter()
        .find(|p| p.name == "Fire")
        .ok_or("Fire")?;
    assert_eq!(
        fire.locals
            .iter()
            .filter(|local| matches!(local.role, ir::SlotRole::InstanceField(_)))
            .count(),
        256
    );
    Ok(())
}
