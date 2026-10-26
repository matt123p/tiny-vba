use std::{collections::BTreeSet, error::Error};
use vba_diagnostics::{SourceFile, SourceId, StatementId};
use vba_ir::*;
use vba_values::{ObjectRef, ScalarKind, Value, VbaString};
type TestResult<T = ()> = Result<T, Box<dyn Error>>;
type ProgramMutation = Box<dyn Fn(&mut Program) -> TestResult>;
type LimitMutation = Box<dyn Fn(&mut VerifyLimits)>;
fn base() -> TestResult<Program> {
    let source = SourceFile::new(
        SourceId(9),
        "manual.bas".into(),
        "Sub Main()\nx=1\nEnd Sub\n".into(),
    )?;
    let declaration = source.span(0, 21)?;
    let assignment = source.span(11, 14)?;
    let end = source.span(15, 22)?;
    // Procedure ownership covers the complete End Sub span.
    let declaration = source.span(declaration.start(), 22)?;
    let a = StatementId {
        source: source.id(),
        ordinal: 0,
    };
    let b = StatementId {
        source: source.id(),
        ordinal: 1,
    };
    let proc = Procedure {
        source: SourceIndex(0),
        name: "Main".into(),
        declaration,
        end_statement: b,
        kind: ProcedureKind::Sub,
        parameters: vec![],
        locals: vec![Local {
            name: "x".into(),
            kind: ScalarKind::Long,
            role: SlotRole::User,
            span: Some(assignment),
            visible: true,
        }],
        result: None,
        entry: Pc(0),
        max_stack: 2,
        code: vec![
            Instruction::Checkpoint(SiteId(0)),
            Instruction::RefLocal(LocalSlot(0)),
            Instruction::Const(ConstantId(0)),
            Instruction::WriteRef(AssignmentKind::Value),
            Instruction::Jump(Pc(5)),
            Instruction::Checkpoint(SiteId(1)),
            Instruction::Return,
        ],
        statements: vec![
            Statement {
                id: a,
                span: assignment,
            },
            Statement { id: b, span: end },
        ],
        checkpoints: vec![
            Checkpoint {
                statement: a,
                span: assignment,
                role: SiteRole::Source,
                visible: true,
                enter_pc: Pc(0),
                resume_next: Some(SiteId(1)),
            },
            Checkpoint {
                statement: b,
                span: end,
                role: SiteRole::SyntheticExit,
                visible: false,
                enter_pc: Pc(5),
                resume_next: None,
            },
        ],
        calls: vec![],
        foreign: vec![],
    };
    Ok(Program {
        ir_version: 1,
        required_features: BTreeSet::from(["scalar-core".into()]),
        profile: "ir1-structural".into(),
        revision: "test-1".into(),
        entry: ProcedureId(0),
        launch: LaunchSignature { arguments: vec![] },
        sources: vec![Source {
            module: "Manual".into(),
            file: source,
        }],
        constants: vec![Value::Long(1)],
        procedures: vec![proc],
    })
}
fn proc_mut(p: &mut Program) -> TestResult<&mut Procedure> {
    p.procedures.first_mut().ok_or_else(|| "procedure".into())
}
fn replace(p: &mut Program, pc: usize, instruction: Instruction) -> TestResult {
    *proc_mut(p)?.code.get_mut(pc).ok_or("instruction")? = instruction;
    Ok(())
}
fn check(p: Program) -> Result<VerifiedProgram, vba_diagnostics::Diagnostic> {
    let caps = if p.ir_version == IR2_VERSION {
        Capabilities::ir2_structural()
    } else {
        Capabilities::ir1_structural()
    };
    verify(p, &caps, &VerifyLimits::default())
}
#[test]
fn malformed_indexes_blocks_stacks_and_metadata_are_rejected() -> TestResult {
    let cases: Vec<ProgramMutation> = vec![
        Box::new(|p| {
            p.entry = ProcedureId(99);
            Ok(())
        }),
        Box::new(|p| {
            proc_mut(p)?.source = SourceIndex(99);
            Ok(())
        }),
        Box::new(|p| replace(p, 2, Instruction::Const(ConstantId(99)))),
        Box::new(|p| replace(p, 1, Instruction::ReadLocal(LocalSlot(99)))),
        Box::new(|p| replace(p, 0, Instruction::Checkpoint(SiteId(99)))),
        Box::new(|p| replace(p, 4, Instruction::Jump(Pc(2)))),
        Box::new(|p| replace(p, 4, Instruction::Jump(Pc(99)))),
        Box::new(|p| replace(p, 4, Instruction::ReadLocal(LocalSlot(0)))),
        Box::new(|p| replace(p, 6, Instruction::DropValue)),
        Box::new(|p| replace(p, 1, Instruction::DropValue)),
        Box::new(|p| replace(p, 1, Instruction::ReadLocal(LocalSlot(0)))),
        Box::new(|p| replace(p, 3, Instruction::DropValue)),
        Box::new(|p| replace(p, 1, Instruction::Return)),
        Box::new(|p| {
            proc_mut(p)?.max_stack = 1;
            Ok(())
        }),
        Box::new(|p| {
            proc_mut(p)?.entry = Pc(1);
            Ok(())
        }),
        Box::new(|p| {
            proc_mut(p)?.code.clear();
            Ok(())
        }),
        Box::new(|p| {
            proc_mut(p)?.checkpoints.first_mut().ok_or("site")?.enter_pc = Pc(2);
            Ok(())
        }),
        Box::new(|p| {
            proc_mut(p)?
                .checkpoints
                .first_mut()
                .ok_or("site")?
                .resume_next = Some(SiteId(99));
            Ok(())
        }),
        Box::new(|p| {
            proc_mut(p)?
                .checkpoints
                .first_mut()
                .ok_or("site")?
                .statement
                .ordinal = 99;
            Ok(())
        }),
        Box::new(|p| {
            proc_mut(p)?
                .checkpoints
                .first_mut()
                .ok_or("site")?
                .statement
                .source = SourceId(44);
            Ok(())
        }),
        Box::new(|p| {
            let source = &p.sources.first().ok_or("source")?.file;
            let s = source.span(11, 11)?;
            proc_mut(p)?.checkpoints.first_mut().ok_or("site")?.span = s;
            Ok(())
        }),
        Box::new(|p| {
            proc_mut(p)?.locals.first_mut().ok_or("slot")?.role = SlotRole::Result;
            Ok(())
        }),
        Box::new(|p| {
            proc_mut(p)?.kind = ProcedureKind::Function;
            Ok(())
        }),
        Box::new(|p| {
            p.sources.push(p.sources.first().ok_or("source")?.clone());
            Ok(())
        }),
        Box::new(|p| {
            p.procedures
                .push(p.procedures.first().ok_or("procedure")?.clone());
            Ok(())
        }),
        Box::new(|p| {
            proc_mut(p)?.name = "x".repeat(257);
            Ok(())
        }),
        Box::new(|p| {
            p.constants = vec![Value::Missing];
            Ok(())
        }),
        Box::new(|p| {
            p.constants = vec![Value::Double(f64::NAN)];
            Ok(())
        }),
        Box::new(|p| {
            p.constants = vec![Value::Object(ObjectRef {
                host: 1,
                object: 1,
                generation: 1,
            })];
            Ok(())
        }),
        Box::new(|p| {
            proc_mut(p)?.calls.push(CallSite {
                checkpoint: SiteId(0),
                callee: ProcedureId(99),
                arguments: vec![],
            });
            Ok(())
        }),
    ];
    for (index, case) in cases.into_iter().enumerate() {
        let mut p = base()?;
        case(&mut p)?;
        assert!(check(p).is_err(), "mutation {index} accepted");
    }
    Ok(())
}
#[test]
fn reachable_infinite_cycle_is_valid_and_unreachable_block_is_rejected() -> TestResult {
    let mut p = base()?;
    let proc = proc_mut(&mut p)?;
    proc.code = vec![Instruction::Checkpoint(SiteId(0)), Instruction::Jump(Pc(0))];
    proc.checkpoints.truncate(1);
    proc.checkpoints.first_mut().ok_or("site")?.resume_next = None;
    proc.max_stack = 0;
    check(p.clone())?;
    let proc = proc_mut(&mut p)?;
    let mut cp = proc.checkpoints.first().ok_or("site")?.clone();
    cp.enter_pc = Pc(2);
    proc.checkpoints.push(cp);
    proc.code
        .extend([Instruction::Checkpoint(SiteId(1)), Instruction::Return]);
    assert!(check(p).is_err());
    Ok(())
}
#[test]
fn feature_profile_version_opcode_operator_and_conversion_gates_are_checked() -> TestResult {
    let mut p = base()?;
    p.ir_version = 3;
    assert_eq!(check(p).err().ok_or("reject")?.code, "VBA0022");
    let mut p = base()?;
    p.required_features.insert("unknown-feature".into());
    assert_eq!(check(p).err().ok_or("reject")?.code, "VBA0022");
    let mut p = base()?;
    p.profile = "other".into();
    assert_eq!(check(p).err().ok_or("reject")?.code, "VBA0022");
    let mut caps = Capabilities::ir1_structural();
    caps.instructions.remove(&Opcode::WriteRef);
    assert_eq!(
        verify(base()?, &caps, &VerifyLimits::default())
            .err()
            .ok_or("reject")?
            .code,
        "VBA0022"
    );
    for instruction in [
        Instruction::Unary(UnaryOp::Negate),
        Instruction::Convert(ScalarKind::Long),
    ] {
        let mut p = base()?;
        let proc = proc_mut(&mut p)?;
        proc.code.insert(3, instruction);
        proc.checkpoints.get_mut(1).ok_or("site")?.enter_pc = Pc(6);
        *proc.code.get_mut(5).ok_or("jump")? = Instruction::Jump(Pc(6));
        assert_eq!(check(p.clone()).err().ok_or("reject")?.code, "VBA0022");
        p.required_features.insert("integer-long-ops".into());
        check(p)?;
    }
    Ok(())
}
#[test]
fn all_binary_and_unary_tags_and_stack_instructions_have_structural_evidence() -> TestResult {
    for op in BINARY_OPS {
        let mut p = base()?;
        p.required_features.insert("integer-long-ops".into());
        let proc = proc_mut(&mut p)?;
        proc.code.splice(
            3..3,
            [Instruction::Const(ConstantId(0)), Instruction::Binary(op)],
        );
        *proc.code.get_mut(6).ok_or("jump")? = Instruction::Jump(Pc(7));
        proc.checkpoints.get_mut(1).ok_or("site")?.enter_pc = Pc(7);
        proc.max_stack = 3;
        check(p.clone())?;
        let mut caps = Capabilities::ir1_structural();
        caps.binary.remove(&op);
        assert!(verify(p, &caps, &VerifyLimits::default()).is_err());
    }
    let mut p = base()?;
    p.required_features.insert("integer-long-ops".into());
    let proc = proc_mut(&mut p)?;
    proc.code = vec![
        Instruction::Checkpoint(SiteId(0)),
        Instruction::RefLocal(LocalSlot(0)),
        Instruction::ReadRef,
        Instruction::DupValue,
        Instruction::DropValue,
        Instruction::Unary(UnaryOp::Not),
        Instruction::DropValue,
        Instruction::Jump(Pc(8)),
        Instruction::Checkpoint(SiteId(1)),
        Instruction::Return,
    ];
    proc.checkpoints.get_mut(1).ok_or("site")?.enter_pc = Pc(8);
    proc.max_stack = 2;
    check(p)?;
    Ok(())
}
fn direct() -> TestResult<Program> {
    let mut p = base()?;
    p.required_features
        .extend(["direct-calls".into(), "byref-locals".into()]);
    let source = &p.sources.first().ok_or("source")?.file;
    let span = source.span(0, 10)?;
    let statement = StatementId {
        source: source.id(),
        ordinal: 10,
    };
    let callee = Procedure {
        source: SourceIndex(0),
        name: "Increment".into(),
        declaration: span,
        end_statement: statement,
        kind: ProcedureKind::Sub,
        parameters: vec![Parameter {
            name: "n".into(),
            kind: ScalarKind::Long,
            mode: ParameterMode::ByRef,
            required: true,
        }],
        locals: vec![Local {
            name: "n".into(),
            kind: ScalarKind::Long,
            role: SlotRole::Parameter(0),
            span: Some(span),
            visible: true,
        }],
        result: None,
        entry: Pc(0),
        max_stack: 0,
        code: vec![Instruction::Checkpoint(SiteId(0)), Instruction::Return],
        statements: vec![Statement {
            id: statement,
            span,
        }],
        checkpoints: vec![Checkpoint {
            statement,
            span,
            role: SiteRole::SyntheticExit,
            visible: false,
            enter_pc: Pc(0),
            resume_next: None,
        }],
        calls: vec![],
        foreign: vec![],
    };
    p.procedures.push(callee);
    let proc = proc_mut(&mut p)?;
    proc.code = vec![
        Instruction::Checkpoint(SiteId(0)),
        Instruction::RefLocal(LocalSlot(0)),
        Instruction::Call {
            procedure: ProcedureId(1),
            site: CallSiteId(0),
        },
        Instruction::Jump(Pc(4)),
        Instruction::Checkpoint(SiteId(1)),
        Instruction::Return,
    ];
    proc.max_stack = 1;
    proc.checkpoints.get_mut(1).ok_or("site")?.enter_pc = Pc(4);
    proc.calls.push(CallSite {
        checkpoint: SiteId(0),
        callee: ProcedureId(1),
        arguments: vec![Argument {
            mode: ArgumentMode::Reference,
            name: None,
            parameter: Some(0),
        }],
    });
    Ok(p)
}
#[test]
fn direct_call_alias_shape_mapping_and_launch_rules_are_verified() -> TestResult {
    check(direct()?)?;
    let cases: Vec<ProgramMutation> = vec![
        Box::new(|p| {
            replace(
                p,
                2,
                Instruction::Call {
                    procedure: ProcedureId(99),
                    site: CallSiteId(0),
                },
            )
        }),
        Box::new(|p| {
            replace(
                p,
                2,
                Instruction::Call {
                    procedure: ProcedureId(1),
                    site: CallSiteId(99),
                },
            )
        }),
        Box::new(|p| {
            proc_mut(p)?.calls.first_mut().ok_or("call")?.checkpoint = SiteId(1);
            Ok(())
        }),
        Box::new(|p| {
            proc_mut(p)?
                .calls
                .first_mut()
                .and_then(|c| c.arguments.first_mut())
                .ok_or("arg")?
                .parameter = Some(1);
            Ok(())
        }),
        Box::new(|p| {
            proc_mut(p)?
                .calls
                .first_mut()
                .and_then(|c| c.arguments.first_mut())
                .ok_or("arg")?
                .name = Some("n".into());
            Ok(())
        }),
        Box::new(|p| {
            proc_mut(p)?
                .calls
                .first_mut()
                .and_then(|c| c.arguments.first_mut())
                .ok_or("arg")?
                .mode = ArgumentMode::Omitted;
            Ok(())
        }),
        Box::new(|p| {
            proc_mut(p)?.locals.first_mut().ok_or("slot")?.kind = ScalarKind::Integer;
            Ok(())
        }),
        Box::new(|p| {
            p.entry = ProcedureId(1);
            p.launch.arguments = vec![ScalarKind::Long];
            Ok(())
        }),
        Box::new(|p| {
            p.procedures
                .get_mut(1)
                .and_then(|p| p.parameters.first_mut())
                .ok_or("parameter")?
                .required = false;
            Ok(())
        }),
    ];
    for (index, case) in cases.into_iter().enumerate() {
        let mut p = direct()?;
        case(&mut p)?;
        assert!(check(p).is_err(), "call mutation {index}");
    }
    Ok(())
}
#[test]
fn foreign_argument_modes_all_kinds_and_stop_are_structurally_verified() -> TestResult {
    for kind in [
        ForeignKind::Call,
        ForeignKind::Get,
        ForeignKind::Let,
        ForeignKind::Set,
    ] {
        let mut p = base()?;
        p.required_features
            .extend(["foreign-snapshot-v1".into(), "debug-stop".into()]);
        let proc = proc_mut(&mut p)?;
        proc.foreign = vec![ForeignSite {
            checkpoint: SiteId(0),
            kind,
            member: "Member".into(),
            arguments: vec![
                Argument {
                    mode: ArgumentMode::Value,
                    name: None,
                    parameter: None,
                },
                Argument {
                    mode: ArgumentMode::Reference,
                    name: None,
                    parameter: None,
                },
                Argument {
                    mode: ArgumentMode::Omitted,
                    name: Some("optional".into()),
                    parameter: None,
                },
            ],
        }];
        proc.code = vec![
            Instruction::Checkpoint(SiteId(0)),
            Instruction::ReadLocal(LocalSlot(0)),
            Instruction::Const(ConstantId(0)),
            Instruction::RefLocal(LocalSlot(0)),
            Instruction::Invoke(ForeignSiteId(0)),
            Instruction::DropValue,
            Instruction::Stop,
            Instruction::Jump(Pc(8)),
            Instruction::Checkpoint(SiteId(1)),
            Instruction::Return,
        ];
        proc.max_stack = 3;
        proc.checkpoints.get_mut(1).ok_or("site")?.enter_pc = Pc(8);
        check(p.clone())?;
        replace(&mut p, 7, Instruction::Return)?;
        assert!(check(p).is_err());
    }
    Ok(())
}
#[test]
fn all_verifier_resource_dimensions_are_enforced() -> TestResult {
    let cases: Vec<LimitMutation> = vec![
        Box::new(|l| l.sources = 0),
        Box::new(|l| l.source_bytes = 1),
        Box::new(|l| l.procedures = 0),
        Box::new(|l| l.instructions_per_procedure = 1),
        Box::new(|l| l.instructions = 1),
        Box::new(|l| l.constants = 0),
        Box::new(|l| l.constant_bytes = 1),
        Box::new(|l| l.slots = 0),
        Box::new(|l| l.stack = 1),
        Box::new(|l| l.metadata_bytes = 1),
        Box::new(|l| l.work = 1),
    ];
    for case in cases {
        let mut limits = VerifyLimits::default();
        case(&mut limits);
        assert_eq!(
            verify(base()?, &Capabilities::ir1_structural(), &limits)
                .err()
                .ok_or("reject")?
                .code,
            "VBA0021"
        );
    }
    Ok(())
}
#[test]
fn exact_scalar_disassembly_preserves_bits_and_utf16() -> TestResult {
    let mut p = base()?;
    p.constants.extend([
        Value::Single(-0.0),
        Value::Double(-0.0),
        Value::Currency(12345),
        Value::Date(2.5),
        Value::String(VbaString::from_units(vec![0xd800, 0x41])?),
        Value::Empty,
        Value::Null,
        Value::Nothing,
        Value::CvErr(2042),
    ]);
    let text = disassemble(&check(p)?)?;
    for expected in [
        "Single(bits=0x80000000)",
        "Double(bits=0x8000000000000000)",
        "Currency(scaled=12345)",
        "Date(serial=2.5",
        r"\ud800\u0041",
        "Empty",
        "Null",
        "Nothing",
        "CvErr(2042)",
    ] {
        assert!(text.contains(expected), "missing {expected}");
    }
    Ok(())
}

#[test]
fn damaged_foreign_descriptors_and_missing_target_operands_are_rejected() -> TestResult {
    let mut p = base()?;
    p.required_features.insert("foreign-snapshot-v1".into());
    let proc = proc_mut(&mut p)?;
    proc.foreign = vec![ForeignSite {
        checkpoint: SiteId(0),
        kind: ForeignKind::Call,
        member: "Call".into(),
        arguments: vec![],
    }];
    proc.code = vec![
        Instruction::Checkpoint(SiteId(0)),
        Instruction::ReadLocal(LocalSlot(0)),
        Instruction::Invoke(ForeignSiteId(0)),
        Instruction::DropValue,
        Instruction::Jump(Pc(5)),
        Instruction::Checkpoint(SiteId(1)),
        Instruction::Return,
    ];
    proc.max_stack = 1;
    check(p.clone())?;
    let cases: Vec<ProgramMutation> = vec![
        Box::new(|p| replace(p, 2, Instruction::Invoke(ForeignSiteId(99)))),
        Box::new(|p| replace(p, 1, Instruction::RefLocal(LocalSlot(0)))),
        Box::new(|p| {
            proc_mut(p)?
                .foreign
                .first_mut()
                .ok_or("foreign")?
                .checkpoint = SiteId(1);
            Ok(())
        }),
        Box::new(|p| {
            proc_mut(p)?.foreign.first_mut().ok_or("foreign")?.member = String::new();
            Ok(())
        }),
        Box::new(|p| {
            proc_mut(p)?.foreign.first_mut().ok_or("foreign")?.arguments = vec![Argument {
                name: None,
                parameter: Some(0),
                mode: ArgumentMode::Omitted,
            }];
            Ok(())
        }),
        Box::new(|p| {
            proc_mut(p)?.foreign.first_mut().ok_or("foreign")?.arguments = vec![
                Argument {
                    name: Some("n".into()),
                    parameter: None,
                    mode: ArgumentMode::Omitted,
                },
                Argument {
                    name: None,
                    parameter: None,
                    mode: ArgumentMode::Omitted,
                },
            ];
            Ok(())
        }),
    ];
    for case in cases {
        let mut invalid = p.clone();
        case(&mut invalid)?;
        assert!(check(invalid).is_err());
    }
    Ok(())
}

#[test]
fn ir2_intrinsic_handler_array_and_persistent_shapes_are_verified() -> TestResult {
    let cases = [
        Instruction::Intrinsic {
            function: vba_values::Intrinsic::Len,
            arguments: 0,
        },
        Instruction::Intrinsic {
            function: vba_values::Intrinsic::Len,
            arguments: 1,
        },
        Instruction::OnError {
            target: Some(Pc(999)),
            resume_next: false,
        },
        Instruction::OnError {
            target: Some(Pc(0)),
            resume_next: true,
        },
        Instruction::Resume {
            target: Some(Pc(1)),
            next: false,
        },
        Instruction::ErrGet("unknown".into()),
        Instruction::Raise(0),
        Instruction::RefElement {
            rank: 0,
            kind: ScalarKind::Long,
        },
        Instruction::RefElement {
            rank: 9,
            kind: ScalarKind::Long,
        },
        Instruction::ArrayResize {
            rank: 9,
            kind: ScalarKind::Long,
            preserve: false,
            initialize: true,
            dynamic: true,
        },
        Instruction::ArrayResize {
            rank: 0,
            kind: ScalarKind::Long,
            preserve: true,
            initialize: true,
            dynamic: true,
        },
        Instruction::ArrayErase,
    ];
    for instruction in cases {
        let mut program = base()?;
        program.ir_version = IR2_VERSION;
        program.profile = "ir2-structural".into();
        program
            .required_features
            .extend(FEATURES.iter().map(|s| (*s).into()));
        *proc_mut(&mut program)?
            .code
            .get_mut(1)
            .ok_or("instruction")? = instruction.clone();
        assert!(check(program).is_err(), "accepted {instruction:?}");
    }
    let mut program = base()?;
    program.ir_version = IR2_VERSION;
    program.profile = "ir2-structural".into();
    program.required_features.insert("persistent-v2".into());
    proc_mut(&mut program)?
        .locals
        .first_mut()
        .ok_or("local")?
        .role = SlotRole::Persistent(0);
    check(program.clone())?;
    proc_mut(&mut program)?
        .locals
        .first_mut()
        .ok_or("local")?
        .role = SlotRole::Persistent(4096);
    assert!(check(program).is_err());
    let mut program = base()?;
    *proc_mut(&mut program)?
        .code
        .get_mut(1)
        .ok_or("instruction")? = Instruction::ErrClear;
    assert!(check(program).is_err());
    Ok(())
}
