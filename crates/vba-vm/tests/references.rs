//! Independently authored host enum/constant binding proofs, without Office names.
use vba_compiler::{CompileOptions, compile};
use vba_diagnostics::{SourceFile, SourceId};
use vba_foreign::{
    ReferenceConstant, ReferenceLibrary, ReferenceSet, ReferenceType, ReferenceTypeKind,
};
use vba_values::Value;
use vba_vm::{Limits, NoHost, Outcome, ReturnValue, Session};
type R<T = ()> = Result<T, Box<dyn std::error::Error>>;
fn options() -> R<CompileOptions> {
    Ok(CompileOptions {
        references: ReferenceSet::new(vec![ReferenceLibrary {
            name: "HostLib".into(),
            version: "1.0".into(),
            available: true,
            types: vec![ReferenceType {
                name: "Mode".into(),
                kind: ReferenceTypeKind::Enum,
            }],
            constants: vec![
                ReferenceConstant {
                    name: "First".into(),
                    module: Some("Modes".into()),
                    value: Value::Long(2),
                },
                ReferenceConstant {
                    name: "Second".into(),
                    module: Some("Modes".into()),
                    value: Value::Long(3),
                },
            ],
        }])?,
        capabilities: vba_vm::capabilities(),
        ..Default::default()
    })
}
fn source(text: &str) -> R<SourceFile> {
    Ok(SourceFile::new(
        SourceId(1),
        "References.bas".into(),
        text.into(),
    )?)
}
fn run(text: &str) -> R<Value> {
    let mut session = Session::new(
        compile(&source(text)?, &options()?)?,
        1,
        vec![],
        None,
        Limits::default(),
    )?;
    for _ in 0..100 {
        match session.run_slice(&mut NoHost, 16, &mut |_| Ok(false))? {
            Outcome::Yielded => (),
            Outcome::Completed(ReturnValue::Function(value)) => return Ok(value),
            other => return Err(format!("unexpected outcome {other:?}").into()),
        }
    }
    Err("execution budget".into())
}
#[test]
fn enum_declarations_parameters_results_and_constants_execute_without_host_dispatch() -> R {
    assert_eq!(
        run(
            "Option Explicit\nFunction Answer() As HostLib.Mode\nDim value As Mode\nvalue = hostlib.modes.first\nBump value\nAnswer = value\nEnd Function\nSub Bump(ByRef value As HostLib.Mode)\nvalue = value + Second\nEnd Sub"
        )?,
        Value::Long(5)
    );
    Ok(())
}
#[test]
fn constants_bind_in_defaults_bounds_and_constant_expressions() -> R {
    assert_eq!(
        run(
            "Const Base As HostLib.Mode = HostLib.First + Second\nFunction Answer() As Long\nDim items(1 To HostLib.Second) As HostLib.Mode\nConst LocalValue As Long = Base + First\nitems(HostLib.First) = Pick()\nAnswer = items(2) + LocalValue\nEnd Function\nFunction Pick(Optional ByVal value As HostLib.Mode = HostLib.Modes.Second) As Long\nPick = value\nEnd Function"
        )?,
        Value::Long(10)
    );
    Ok(())
}
#[test]
fn local_and_module_symbols_shadow_library_constants() -> R {
    assert_eq!(
        run(
            "Const First As Long = 11\nFunction Answer() As Long\nDim Second As Long\nSecond = 7\nAnswer = First + Second + HostLib.Second\nEnd Function"
        )?,
        Value::Long(21)
    );
    assert!(compile(&source("Function Answer() As Long\nDim HostLib As Long\nAnswer = HostLib.First\nEnd Function")?, &options()?).is_err());
    Ok(())
}
#[test]
fn reference_constants_cannot_be_written_or_called() -> R {
    for statement in [
        "First = 4",
        "HostLib.First = 4",
        "HostLib.Modes.First = 4",
        "Call HostLib.First()",
    ] {
        assert!(
            compile(
                &source(&format!("Sub Answer()\n{statement}\nEnd Sub"))?,
                &options()?
            )
            .is_err(),
            "{statement}"
        );
    }
    Ok(())
}
#[test]
fn unregistered_types_and_missing_libraries_are_explicit_diagnostics() -> R {
    let file = source("Function Answer() As Unknown.Document\nSet Answer = Nothing\nEnd Function")?;
    let error = compile(&file, &options()?).err().ok_or("diagnostic")?;
    assert_eq!(error.code, "VBA0132");
    assert_eq!(file.slice(error.span.ok_or("span")?)?, "Unknown.Document");
    let mut options = options()?;
    options.references = ReferenceSet::new(vec![ReferenceLibrary {
        name: "MissingLib".into(),
        version: "1".into(),
        available: false,
        types: vec![],
        constants: vec![],
    }])?;
    let file =
        source("Function Answer() As MissingLib.Document\nSet Answer = Nothing\nEnd Function")?;
    assert_eq!(
        compile(&file, &options)
            .err()
            .ok_or("missing diagnostic")?
            .code,
        "VBA0131"
    );
    Ok(())
}
