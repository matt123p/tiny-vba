//! Explicit generic library names lowered to ordinary, bounded source calls.
use vba_compiler::{CompileOptions, compile_modules};
use vba_diagnostics::{SourceFile, SourceId};
use vba_syntax::{ModuleKind, parse};
use vba_values::Value;
use vba_vm::{Limits, NoHost, Outcome, ReturnValue, Session};
type R<T = ()> = Result<T, Box<dyn std::error::Error>>;
fn program(main: &str, provider: &str, options: &CompileOptions) -> R<vba_ir::VerifiedProgram> {
    Ok(compile_modules(
        &[
            parse(
                &SourceFile::new(SourceId(1), "Main.bas".into(), main.into())?,
                ModuleKind::Standard,
            )?,
            parse(
                &SourceFile::new(SourceId(2), "Provider.bas".into(), provider.into())?,
                ModuleKind::Standard,
            )?,
            parse(
                &SourceFile::new(
                    SourceId(3),
                    "Counter.cls".into(),
                    "Public Function Query() As Long\nQuery=9\nEnd Function".into(),
                )?,
                ModuleKind::Class,
            )?,
        ],
        options,
    )?)
}
fn options() -> CompileOptions {
    CompileOptions {
        capabilities: vba_vm::capabilities(),
        procedure_aliases: [
            ("GlobalQuery".into(), ("Provider".into(), "Read".into())),
            ("Lib.Query".into(), ("Provider".into(), "Read".into())),
        ]
        .into_iter()
        .collect(),
        ..Default::default()
    }
}
fn answer(main: &str, provider: &str, expected: i32) -> R {
    let mut s = Session::new(
        program(main, provider, &options())?,
        1,
        vec![],
        None,
        Limits::default(),
    )?;
    for _ in 0..1000 {
        let outcome = s.run_slice(&mut NoHost, 64, &mut |_| Ok(false))?;
        if outcome != Outcome::Yielded {
            assert_eq!(
                outcome,
                Outcome::Completed(ReturnValue::Function(Value::Long(expected)))
            );
            return Ok(());
        }
    }
    Err("timeout".into())
}
const PROVIDER: &str =
    "Public Function Read(Optional ByVal value As Long=3) As Long\nRead=value+2\nEnd Function";
#[test]
fn qualified_and_bare_aliases_preserve_optional_named_arguments_and_call_statement_results() -> R {
    answer(
        "Function Answer() As Long\nAnswer=GlobalQuery+Lib.Query(5)+Lib.Query(value:=7)\nCall Lib.Query()\nEnd Function",
        PROVIDER,
        21,
    )
}
#[test]
fn declared_variables_and_project_functions_shadow_library_aliases() -> R {
    answer(
        "Function Answer() As Long\nAnswer=GlobalQuery+Lib.Query()\nEnd Function\nFunction GlobalQuery() As Long\nGlobalQuery=40\nEnd Function",
        PROVIDER,
        45,
    )?;
    answer(
        "Function Answer() As Long\nDim Lib As Counter\nSet Lib=New Counter\nAnswer=Lib.Query()\nEnd Function",
        PROVIDER,
        9,
    )
}
#[test]
fn malformed_private_ambiguous_or_undeclared_alias_targets_refuse_before_execution() -> R {
    let main = "Function Answer() As Long\nAnswer=1\nEnd Function";
    assert!(
        program(
            main,
            &PROVIDER.replace("Public Function", "Private Function"),
            &options()
        )
        .is_err()
    );
    for (alias, owner, member) in [
        ("A.B.C", "Provider", "Read"),
        ("Lib.Query", "Missing", "Read"),
        ("Lib.Query", "Provider", "Missing"),
        ("_Bad", "Provider", "Read"),
    ] {
        let mut options = options();
        options.procedure_aliases = [(alias.into(), (owner.into(), member.into()))]
            .into_iter()
            .collect();
        assert!(program(main, PROVIDER, &options).is_err());
    }
    let mut duplicate = options();
    duplicate
        .procedure_aliases
        .insert("lib.query".into(), ("Provider".into(), "Read".into()));
    assert!(program(main, PROVIDER, &duplicate).is_err());
    Ok(())
}

#[test]
fn registered_aliases_are_not_implicitly_writable_variables_and_owner_ambiguity_refuses() -> R {
    for statement in ["GlobalQuery=1", "Lib.Query=1"] {
        assert!(
            program(
                &format!("Function Answer() As Long\n{statement}\nAnswer=1\nEnd Function"),
                PROVIDER,
                &options()
            )
            .is_err()
        );
    }
    assert!(
        program(
            "Attribute VB_Name = \"Provider\"\nFunction Answer() As Long\nAnswer=1\nEnd Function",
            PROVIDER,
            &options()
        )
        .is_err()
    );
    Ok(())
}
