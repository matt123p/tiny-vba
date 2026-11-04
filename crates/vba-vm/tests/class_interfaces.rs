//! Generic explicit native-class interface metadata; no Office names or activation.
use vba_compiler::{CompileOptions, compile_modules};
use vba_diagnostics::{SourceFile, SourceId};
use vba_foreign::{ReferenceLibrary, ReferenceSet, ReferenceType, ReferenceTypeKind};
use vba_syntax::{ModuleKind, parse};
use vba_values::{NominalType, Value};
use vba_vm::{Limits, NoHost, Outcome, ReturnValue, Session};
type R<T = ()> = Result<T, Box<dyn std::error::Error>>;
fn options() -> R<CompileOptions> {
    Ok(CompileOptions {
        capabilities: vba_vm::capabilities(),
        references: ReferenceSet::new(vec![ReferenceLibrary {
            name: "HostLib".into(),
            version: "1".into(),
            available: true,
            types: ["Face", "Common", "Other"]
                .into_iter()
                .map(|name| ReferenceType {
                    name: name.into(),
                    kind: ReferenceTypeKind::Object,
                })
                .collect(),
            constants: vec![],
        }])?,
        class_interfaces: [(
            "Counter".into(),
            vec![
                vba_ir::ClassInterface {
                    ty: NominalType::new("HostLib", "1", "Face")?,
                    events: true,
                    event_names: None,
                },
                vba_ir::ClassInterface {
                    ty: NominalType::new("HostLib", "1", "Common")?,
                    events: false,
                    event_names: None,
                },
            ],
        )]
        .into_iter()
        .collect(),
        ..Default::default()
    })
}
fn program(
    main: &str,
    counter: &str,
    sink: &str,
    options: &CompileOptions,
) -> R<vba_ir::VerifiedProgram> {
    let modules = [
        parse(
            &SourceFile::new(SourceId(1), "Main.bas".into(), main.into())?,
            ModuleKind::Standard,
        )?,
        parse(
            &SourceFile::new(SourceId(2), "Counter.cls".into(), counter.into())?,
            ModuleKind::Class,
        )?,
        parse(
            &SourceFile::new(SourceId(3), "Sink.cls".into(), sink.into())?,
            ModuleKind::Class,
        )?,
    ];
    Ok(compile_modules(&modules, options)?)
}
fn session(main: &str, counter: &str, sink: &str) -> R<Session> {
    Ok(Session::new(
        program(main, counter, sink, &options()?)?,
        1,
        vec![],
        None,
        Limits::default(),
    )?)
}
fn drive(s: &mut Session) -> R<Outcome> {
    for _ in 0..1000 {
        let result = s.run_slice(&mut NoHost, 64, &mut |_| Ok(false))?;
        if result != Outcome::Yielded {
            return Ok(result);
        }
    }
    Err("timeout".into())
}
#[test]
fn interface_typed_variables_arrays_parameters_and_results_keep_native_identity() -> R {
    let mut s = session(
        "Function Answer() As Long\nDim c As Counter,f As HostLib.Face,b As HostLib.Common,a(0 To 0) As HostLib.Face\nSet c=New Counter\nSet f=c\nSet b=f\nSet a(0)=b\nTouch f\nIf a(0) Is c Then Answer=Pass(b).value\nEnd Function\nSub Touch(ByRef value As HostLib.Face)\nvalue.value=9\nEnd Sub\nFunction Pass(ByVal value As HostLib.Common) As HostLib.Face\nSet Pass=value\nEnd Function",
        "Public value As Long",
        "",
    )?;
    assert_eq!(
        drive(&mut s)?,
        Outcome::Completed(ReturnValue::Function(Value::Long(9)))
    );
    Ok(())
}
#[test]
fn interface_with_events_uses_native_connections_and_rejects_bad_signatures_before_execution() -> R
{
    let main = "Function Answer() As Long\nDim c As Counter,s As Sink,f As HostLib.Face\nSet c=New Counter\nSet s=New Sink\nSet f=c\ns.Connect f\nc.Fire\nAnswer=s.value\nEnd Function";
    let counter = "Public Event Changed(ByVal amount As Long)\nPublic Sub Fire()\nRaiseEvent Changed(7)\nEnd Sub";
    let sink = "Private WithEvents publisher As HostLib.Face\nPublic value As Long\nPublic Sub Connect(ByVal value As HostLib.Face)\nSet publisher=value\nEnd Sub\nPrivate Sub publisher_Changed(ByVal amount As Long)\nvalue=amount\nEnd Sub";
    let mut s = session(main, counter, sink)?;
    assert_eq!(
        drive(&mut s)?,
        Outcome::Completed(ReturnValue::Function(Value::Long(7)))
    );
    assert!(
        program(
            main,
            counter,
            &sink.replace(
                "publisher_Changed(ByVal amount As Long)",
                "publisher_Changed(ByVal amount As String)"
            ),
            &options()?
        )
        .is_err()
    );
    Ok(())
}
#[test]
fn wrong_interfaces_stay_error_thirteen_and_restore_revalidates_current_session_identity() -> R {
    let mut wrong = session(
        "Function Answer() As Long\nDim c As Counter,o As HostLib.Other\nSet c=New Counter\nOn Error Resume Next\nSet o=c\nAnswer=Err.Number\nEnd Function",
        "Public value As Long",
        "",
    )?;
    assert_eq!(
        drive(&mut wrong)?,
        Outcome::Completed(ReturnValue::Function(Value::Long(13)))
    );
    let p = program(
        "Private f As HostLib.Face\nFunction Answer() As Long\nDim c As Counter\nIf f Is Nothing Then\nSet c=New Counter\nSet f=c\nEnd If\nf.value=f.value+1\nAnswer=f.value\nEnd Function",
        "Public value As Long",
        "",
        &options()?,
    )?;
    let mut first = Session::new(p.clone(), 1, vec![], None, Limits::default())?;
    assert_eq!(
        drive(&mut first)?,
        Outcome::Completed(ReturnValue::Function(Value::Long(1)))
    );
    let old = *first
        .class_instances("Counter")?
        .first()
        .ok_or("native instance")?;
    let saved = first.project_state()?;
    let mut second = Session::new(p, 2, vec![], None, Limits::default())?;
    second.restore_project(saved)?;
    assert_eq!(
        drive(&mut second)?,
        Outcome::Completed(ReturnValue::Function(Value::Long(2)))
    );
    assert!(second.read_instance_field(old, "value").is_err());
    Ok(())
}
#[test]
fn interface_registration_and_legacy_capability_boundaries_are_checked() -> R {
    let main = "Function Answer() As Long\nAnswer=1\nEnd Function";
    let options = options()?;
    let p = program(main, "", "", &options)?;
    assert_eq!(p.program().ir_version, vba_ir::IR11_VERSION);
    let mut legacy = vba_ir::Capabilities::ir10_structural();
    legacy.profile = p.program().profile.clone();
    assert!(vba_ir::recheck(&p, &legacy, &Default::default()).is_err());
    let mut raw = p.program().clone();
    raw.required_features.remove("class-interfaces-v11");
    assert!(vba_ir::verify(raw, &vba_vm::capabilities(), &Default::default()).is_err());
    let mut duplicate = options.clone();
    duplicate
        .class_interfaces
        .get_mut("Counter")
        .ok_or("claims")?
        .push(vba_ir::ClassInterface {
            ty: NominalType::new("HostLib", "1", "Face")?,
            events: true,
            event_names: None,
        });
    assert!(program(main, "", "", &duplicate).is_err());
    let mut unknown = options.clone();
    unknown.class_interfaces.insert("Missing".into(), vec![]);
    assert!(program(main, "", "", &unknown).is_err());
    let mut undeclared = options;
    undeclared.class_interfaces.insert(
        "Counter".into(),
        vec![vba_ir::ClassInterface {
            ty: NominalType::new("OtherLib", "1", "Face")?,
            events: true,
            event_names: None,
        }],
    );
    assert!(program(main, "", "", &undeclared).is_err());
    Ok(())
}

#[test]
fn a_non_event_interface_refuses_connection_before_mutating_the_field() -> R {
    let mut s = session(
        "Function Answer() As Long\nDim c As Counter,s As Sink,f As HostLib.Common\nSet c=New Counter\nSet s=New Sink\nSet f=c\nOn Error Resume Next\ns.Connect f\nAnswer=Err.Number\nEnd Function",
        "Public Event Changed\nPublic Sub Fire()\nRaiseEvent Changed\nEnd Sub",
        "Private WithEvents publisher As HostLib.Common\nPublic Sub Connect(ByVal value As HostLib.Common)\nSet publisher=value\nEnd Sub",
    )?;
    assert_eq!(
        drive(&mut s)?,
        Outcome::Completed(ReturnValue::Function(Value::Long(459)))
    );
    Ok(())
}

#[test]
fn interface_event_exposure_hides_internal_events_and_does_not_bind_lookalike_helpers() -> R {
    let main = "Function Answer() As Long\nDim c As Counter,s As Sink,f As HostLib.Face\nSet c=New Counter\nSet s=New Sink\nSet f=c\ns.Connect f\nc.Fire\nAnswer=s.value\nEnd Function";
    let counter = "Public Event Changed(ByVal amount As Long)\nPublic Event Internal(ByVal data As String)\nPublic Sub Fire()\nRaiseEvent Internal(\"hidden\")\nRaiseEvent Changed(7)\nEnd Sub";
    let sink = "Private WithEvents publisher As HostLib.Face\nPublic value As Long\nPublic Sub Connect(ByVal value As HostLib.Face)\nSet publisher=value\nEnd Sub\nPrivate Sub publisher_Changed(ByVal amount As Long)\nvalue=amount\nEnd Sub\nPrivate Sub publisher_Internal()\nvalue=999\nEnd Sub";
    let mut options = options()?;
    options
        .class_interfaces
        .get_mut("Counter")
        .and_then(|claims| claims.first_mut())
        .ok_or("interface")?
        .event_names = Some(vec!["Changed".into()]);
    let mut s = Session::new(
        program(main, counter, sink, &options)?,
        1,
        vec![],
        None,
        Limits::default(),
    )?;
    assert_eq!(
        drive(&mut s)?,
        Outcome::Completed(ReturnValue::Function(Value::Long(7)))
    );
    options
        .class_interfaces
        .get_mut("Counter")
        .and_then(|claims| claims.first_mut())
        .ok_or("interface")?
        .event_names = Some(vec!["Missing".into()]);
    assert!(program(main, counter, sink, &options).is_err());
    Ok(())
}
