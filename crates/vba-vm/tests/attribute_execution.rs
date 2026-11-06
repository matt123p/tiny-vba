//! Exact recorded-macro attribute shape, independently authored and host-free.
use vba_compiler::{CompileOptions, compile};
use vba_diagnostics::{SourceFile, SourceId};
use vba_values::VbaString;
use vba_vm::{Limits, NoHost, Outcome, ReturnValue, Session};
type R<T = ()> = Result<T, Box<dyn std::error::Error>>;
#[test]
fn recorded_macro_attribute_has_no_executable_checkpoint() -> R {
    let source = "Attribute VB_Name = \"Module1\"\nSub Macro2()\nAttribute Macro2.VB_ProcData.VB_Invoke_Func = \" \\n14\"\nDebug.Print 42\nEnd Sub\n";
    let options = CompileOptions {
        entry: "Macro2".into(),
        capabilities: vba_vm::capabilities(),
        ..Default::default()
    };
    let program = compile(
        &SourceFile::new(SourceId(1), "Module1.bas".into(), source.into())?,
        &options,
    )?;
    for checkpoint in program
        .program()
        .procedures
        .iter()
        .flat_map(|p| &p.checkpoints)
    {
        assert!(
            !source
                .get(checkpoint.span.start() as usize..checkpoint.span.end() as usize)
                .ok_or("span")?
                .contains("Attribute")
        );
    }
    let mut session = Session::new(program, 1, vec![], None, Limits::default())?;
    let mut completed = false;
    for _ in 0..100 {
        if session.run_slice(&mut NoHost, 64, &mut |_| Ok(false))?
            == Outcome::Completed(ReturnValue::Sub)
        {
            completed = true;
            break;
        }
    }
    assert!(completed);
    assert_eq!(session.output(), &[VbaString::from_utf8("42")?]);
    Ok(())
}

#[test]
fn procedure_and_field_default_member_metadata_execute_or_refuse_explicitly() -> R {
    use vba_syntax::{ModuleKind, parse};
    for class in [
        "Public Property Get Item() As Long\nAttribute Item.VB_UserMemId = +0\nItem=42\nEnd Property\n",
        "Public Function Item() As Long\nAttribute Item.VB_UserMemId = 0\nItem=42\nEnd Function\n",
    ] {
        let main =
            "Function Answer() As Long\nDim box As Box\nSet box=New Box\nAnswer=box\nEnd Function";
        let modules = [
            parse(
                &SourceFile::new(SourceId(1), "Main.bas".into(), main.into())?,
                ModuleKind::Standard,
            )?,
            parse(
                &SourceFile::new(SourceId(2), "Box.cls".into(), class.into())?,
                ModuleKind::Class,
            )?,
        ];
        let program = vba_compiler::compile_modules(
            &modules,
            &CompileOptions {
                capabilities: vba_vm::capabilities(),
                ..Default::default()
            },
        )?;
        let mut session = Session::new(program, 1, vec![], None, Limits::default())?;
        assert_eq!(
            session.run_slice(&mut NoHost, 64, &mut |_| Ok(false))?,
            Outcome::Completed(ReturnValue::Function(vba_values::Value::Long(42)))
        );
    }
    for (class, diagnostic) in [
        (
            "Public Function NewEnum() As Long\nAttribute NewEnum.VB_UserMemId = -4\nEnd Function",
            "enumerator",
        ),
        (
            "Public Item As Long\nAttribute Item.VB_VarUserMemId = 0\n",
            "default",
        ),
        (
            "Private Property Get Item() As Long\nAttribute Item.VB_UserMemId = 0\nItem=42\nEnd Property",
            "default",
        ),
        (
            "Public Sub Item()\nAttribute Item.VB_UserMemId = 0\nEnd Sub",
            "default",
        ),
    ] {
        let modules = [
            parse(
                &SourceFile::new(
                    SourceId(1),
                    "Main.bas".into(),
                    "Function Answer() As Long\nAnswer=1\nEnd Function".into(),
                )?,
                ModuleKind::Standard,
            )?,
            parse(
                &SourceFile::new(SourceId(2), "Box.cls".into(), class.into())?,
                ModuleKind::Class,
            )?,
        ];
        let error = vba_compiler::compile_modules(&modules, &CompileOptions::default())
            .err()
            .ok_or("unsupported metadata must not silently compile")?;
        assert!(
            error.message.to_ascii_lowercase().contains(diagnostic),
            "{error}"
        );
    }
    Ok(())
}
