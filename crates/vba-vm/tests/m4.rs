use std::error::Error;
use vba_compiler::{CompileOptions, compile};
use vba_diagnostics::{SourceFile, SourceId};
use vba_values::{Value, VbaString};
use vba_vm::{self as vm, Limits, NoHost, Outcome, PauseReason, ReturnValue, Session};
type R<T = ()> = Result<T, Box<dyn Error>>;
fn session(text: &str) -> R<Session> {
    let source = SourceFile::new(SourceId(41), "m4.bas".into(), text.into())?;
    let program = compile(
        &source,
        &CompileOptions {
            capabilities: vm::capabilities(),
            ..Default::default()
        },
    )?;
    Ok(Session::new(program, 42, vec![], None, Limits::default())?)
}
fn drive(s: &mut Session) -> R<Outcome> {
    for _ in 0..10000 {
        let out = s.run_slice(&mut NoHost, 7, &mut |_| Ok(false))?;
        if out != Outcome::Yielded {
            return Ok(out);
        }
    }
    Err("slice limit".into())
}
fn answer(source: &str, expected: Value) -> R {
    assert_eq!(
        drive(&mut session(source)?)?,
        Outcome::Completed(ReturnValue::Function(expected)),
        "{source}"
    );
    Ok(())
}
fn number(source: &str, expected: i32) -> R {
    let Outcome::Failed(f) = drive(&mut session(source)?)? else {
        return Err("expected numbered failure".into());
    };
    assert_eq!(f.kind, vm::FailureKind::Runtime);
    assert_eq!(f.number, Some(expected));
    Ok(())
}
#[test]
fn optional_defaults_missing_named_and_parenthesized_byref() -> R {
    answer(
        "Function Answer() As Long\nConst c As Long=2+3\nAnswer=F()+c\nEnd Function\nFunction F(Optional ByVal n As Long=-2) As Long\nF=n\nEnd Function",
        Value::Long(3),
    )?;
    answer(
        "Function Answer() As Long\nAnswer=F(right:=3,left:=2)\nEnd Function\nFunction F(Optional ByVal left As Long=1,Optional ByVal right As Long=4) As Long\nF=left*10+right\nEnd Function",
        Value::Long(23),
    )?;
    answer(
        "Function Answer() As Boolean\nAnswer=F()\nEnd Function\nFunction F(Optional x As Variant) As Boolean\nF=IsMissing(x)\nEnd Function",
        Value::Boolean(true),
    )?;
    answer(
        "Function Answer() As Long\nDim x As Long\nx=4\nCall Inc((x))\nCall Inc(x+1)\nAnswer=x\nEnd Function\nSub Inc(ByRef n As Long)\nn=n+1\nEnd Sub",
        Value::Long(4),
    )?;
    answer(
        "Function Answer() As Long\nAnswer=F(,3)\nEnd Function\nFunction F(Optional ByVal a As Long=2,Optional ByVal b As Long=4) As Long\nF=a*10+b\nEnd Function",
        Value::Long(23),
    )
}
#[test]
fn named_values_evaluate_in_source_order_once() -> R {
    answer(
        "Dim n As Long\nFunction Answer() As Long\nAnswer=Compose(b:=Tick(),a:=Tick())\nEnd Function\nFunction Tick() As Long\nn=n+1\nTick=n\nEnd Function\nFunction Compose(ByVal a As Long,ByVal b As Long) As Long\nCompose=a*10+b\nEnd Function",
        Value::Long(21),
    )
}
#[test]
fn resume_next_clears_partial_stacks_and_preserves_prior_effects() -> R {
    answer(
        "Function Answer() As Long\nDim n As Long\nn=5\nOn Error Resume Next\nn=12+1/0\nAnswer=n+Err.Number\nEnd Function",
        Value::Long(16),
    )?;
    answer(
        "Function Answer() As Long\nDim n As Long\nOn Error Resume Next\nCall F(n)\nAnswer=n+Err.Number\nEnd Function\nSub F(ByRef n As Long)\nn=30\nn=1/0\nEnd Sub",
        Value::Long(41),
    )
}
#[test]
fn goto_handler_retry_and_resume_next_do_not_repeat_completed_statements() -> R {
    answer(
        "Function Answer() As Long\nDim divisor As Long\nDim effects As Long\nOn Error GoTo Recover\neffects=effects+1\nAnswer=20/divisor\nAnswer=Answer+effects\nExit Function\nRecover:\ndivisor=2\nResume\nEnd Function",
        Value::Long(11),
    )?;
    answer(
        "Function Answer() As Long\nOn Error GoTo Recover\nAnswer=1/0\nAnswer=Err.Number+1\nExit Function\nRecover:\nResume Next\nEnd Function",
        Value::Long(1),
    )
}
#[test]
fn nested_active_handlers_propagate_and_err_clear_does_not_reenable() -> R {
    answer(
        "Function Answer() As Long\nOn Error GoTo Outer\nCall F()\nExit Function\nOuter:\nAnswer=Err.Number\nEnd Function\nSub F()\nOn Error GoTo Inner\nErr.Raise 12\nExit Sub\nInner:\nErr.Clear\nErr.Raise 17\nEnd Sub",
        Value::Long(17),
    )?;
    number(
        "Function Answer() As Long\nOn Error GoTo Recover\nAnswer=1/0\nExit Function\nRecover:\nErr.Clear\nAnswer=1/0\nEnd Function",
        11,
    )
}
#[test]
fn disable_handler_and_resume_without_fault_are_numbered() -> R {
    number(
        "Function Answer() As Long\nOn Error Resume Next\nOn Error GoTo 0\nAnswer=1/0\nEnd Function",
        11,
    )?;
    number("Function Answer() As Long\nResume Next\nEnd Function", 20)
}
#[test]
fn err_raise_properties_and_numeric_line_labels() -> R {
    answer(
        "Function Answer() As Long\nOn Error GoTo Recover\n120 Answer=1/0\nExit Function\nRecover:\nAnswer=Erl+Err.Number\nEnd Function",
        Value::Long(131),
    )?;
    answer(
        "Function Answer() As String\nOn Error GoTo Recover\nErr.Raise 513,\"test-source\",\"test-message\"\nExit Function\nRecover:\nAnswer=Err.Source & \":\" & Err.Description\nEnd Function",
        Value::String(VbaString::from_utf8("test-source:test-message")?),
    )
}
#[test]
fn fault_pause_continues_into_handler_once_and_invalidates_handles() -> R {
    let mut s = session(
        "Function Answer() As Long\nOn Error GoTo Recover\nAnswer=1/0\nExit Function\nRecover:\nAnswer=Err.Number\nEnd Function",
    )?;
    s.set_break_on_error(true);
    let Outcome::Paused(p) = drive(&mut s)? else {
        return Err("fault pause".into());
    };
    assert_eq!(p.reason, PauseReason::Fault);
    let count = s.counters();
    s.continue_execution()?;
    assert_eq!(s.counters(), count);
    assert_eq!(
        drive(&mut s)?,
        Outcome::Completed(ReturnValue::Function(Value::Long(11)))
    );
    assert!(s.read_local(p.handle, "Answer").is_err());
    Ok(())
}
#[test]
fn structured_header_resume_next_skips_faulting_control_statement() -> R {
    answer(
        "Function Answer() As Long\nOn Error Resume Next\nIf 1/0 Then\nAnswer=99\nEnd If\nAnswer=Err.Number\nEnd Function",
        Value::Long(11),
    )?;
    answer(
        "Function Answer() As Long\nDim i As Long\nOn Error Resume Next\nFor i=1 To 1/0\nAnswer=99\nNext i\nAnswer=Err.Number\nEnd Function",
        Value::Long(11),
    )
}
#[test]
fn arrays_preserve_lower_bounds_column_order_and_aliases() -> R {
    answer(
        include_str!("../../../examples/pure-vba/M4Arrays.bas"),
        Value::Long(44),
    )?;
    answer(
        "Function Answer() As Long\nDim a(-1 To 1) As Long\na(-1)=5\nCall F(a(-1),a(-1))\nAnswer=a(-1)\nEnd Function\nSub F(ByRef x As Long,ByRef y As Long)\nx=x+1\ny=y+x\nEnd Sub",
        Value::Long(12),
    )
}
#[test]
fn erase_reinitializes_fixed_and_unallocates_dynamic_arrays() -> R {
    answer(
        "Function Answer() As Long\nDim a(2 To 3) As Long\na(2)=7\nErase a\nAnswer=a(2)+LBound(a)\nEnd Function",
        Value::Long(2),
    )?;
    number(
        "Function Answer() As Long\nDim a() As Long\nReDim a(2)\nErase a\nAnswer=UBound(a)\nEnd Function",
        9,
    )
}
#[test]
fn array_errors_are_catchable_and_preserve_original_storage() -> R {
    answer(
        "Function Answer() As Long\nDim a() As Long\nReDim a(1 To 2,3 To 4)\na(1,3)=9\nOn Error Resume Next\nReDim Preserve a(1 To 3,3 To 4)\nAnswer=a(1,3)+Err.Number\nEnd Function",
        Value::Long(18),
    )?;
    number(
        "Function Answer() As Long\nDim a(2) As Long\nAnswer=a(3)\nEnd Function",
        9,
    )
}
#[test]
fn project_static_lifetime_restore_reset_and_source_identity() -> R {
    let text = include_str!("../../../examples/pure-vba/M4State.bas");
    let mut s = session(text)?;
    assert_eq!(
        drive(&mut s)?,
        Outcome::Completed(ReturnValue::Function(Value::Long(23)))
    );
    let state = s.project_state()?;
    let mut restored = session(text)?;
    restored.restore_project(state.clone())?;
    assert_eq!(
        drive(&mut restored)?,
        Outcome::Completed(ReturnValue::Function(Value::Long(47)))
    );
    assert!(restored.restore_project(state.clone()).is_err());
    let mut reset = session(text)?;
    assert_eq!(
        drive(&mut reset)?,
        Outcome::Completed(ReturnValue::Function(Value::Long(23)))
    );
    assert!(
        session(&text.replace("n+10", "n+11"))?
            .restore_project(state)
            .is_err()
    );
    Ok(())
}
#[test]
fn exact_numeric_tags_null_states_and_intrinsics_execute() -> R {
    for (body, ty, value) in [
        ("CCur(1.25)+CCur(0.5)", "Currency", Value::Currency(17500)),
        ("CDate(45000)+2", "Date", Value::Date(45002.0)),
        ("CSng(1.5)*CSng(2)", "Single", Value::Single(3.0)),
        (
            "CLngLng(\"9007199254740993\")+CLngLng(1)",
            "LongLong",
            Value::LongLong(9007199254740994),
        ),
        ("CByte(255)", "Byte", Value::Byte(255)),
        (
            "IsError(CVErr(2042)) And IsEmpty(Empty) And IsNull(Null)",
            "Boolean",
            Value::Boolean(true),
        ),
        ("Null+1", "Variant", Value::Null),
        (
            "Mid(\"abcdef\",2,3)",
            "String",
            Value::String(VbaString::from_utf8("bcd")?),
        ),
        (
            "Left(Trim(\" abcd \"),2) & Right(\"xyz\",2)",
            "String",
            Value::String(VbaString::from_utf8("abyz")?),
        ),
    ] {
        answer(
            &format!("Function Answer() As {ty}\nAnswer={body}\nEnd Function"),
            value,
        )?;
    }
    number("Function Answer() As Byte\nAnswer=256\nEnd Function", 6)?;
    number("Function Answer() As Long\nAnswer=Null\nEnd Function", 94)
}
#[test]
fn resource_failure_cannot_be_swallowed_by_on_error() -> R {
    let mut s = session(
        "Function Answer() As Long\nDim a() As Long\nOn Error Resume Next\nReDim a(1000000)\nAnswer=42\nEnd Function",
    )?;
    let Outcome::Failed(f) = drive(&mut s)? else {
        return Err("expected resource failure".into());
    };
    assert_eq!(f.kind, vm::FailureKind::ResourceLimit);
    assert_eq!(f.number, None);
    Ok(())
}

#[test]
fn string_search_replace_split_join_and_captured_print() -> R {
    answer(
        "Function Answer() As String\nAnswer=Join(Split(Replace(\"ab,ab\",\"ab\",\"xy\"),\",\"),\"-\")\nEnd Function",
        Value::String(VbaString::from_utf8("xy-xy")?),
    )?;
    answer(
        "Function Answer() As Long\nAnswer=InStr(2,\"banana\",\"na\")\nEnd Function",
        Value::Long(3),
    )?;
    let mut s =
        session("Function Answer() As Long\nDebug.Print \"one\", 2\nAnswer=3\nEnd Function")?;
    assert_eq!(
        drive(&mut s)?,
        Outcome::Completed(ReturnValue::Function(Value::Long(3)))
    );
    assert_eq!(s.output(), &[VbaString::from_utf8("one\t2")?]);
    Ok(())
}
#[test]
fn stale_element_alias_after_redim_is_numbered_not_an_engine_crash() -> R {
    number(
        "Dim a() As Long\nFunction Answer() As Long\nReDim a(2)\nCall F(a(0))\nEnd Function\nSub F(ByRef x As Long)\nReDim a(3)\nx=1\nEnd Sub",
        9,
    )
}
#[test]
fn unknown_or_duplicate_labels_and_bad_argument_forms_fail_before_execution() -> R {
    for text in [
        "Function Answer() As Long\nGoTo absent\nEnd Function",
        "Function Answer() As Long\nSame:\nSame:\nEnd Function",
        "Function Answer() As Long\nOn Error GoTo absent\nEnd Function",
        "Function Answer() As Long\nResume absent\nEnd Function",
        "Function Answer() As Long\nAnswer=F(a:=1,a:=2)\nEnd Function\nFunction F(ByVal a As Long) As Long\nEnd Function",
    ] {
        assert!(session(text).is_err(), "accepted {text}");
    }
    Ok(())
}

#[test]
fn positional_values_before_evaluated_byref_temporaries_keep_source_order() -> R {
    answer(
        "Function Answer() As Boolean\nDim target As Object\nAnswer=F(target:=target)\nEnd Function\nFunction F(ByVal target As Object) As Boolean\nF=IsObject(target)\nEnd Function",
        Value::Boolean(true),
    )?;
    answer(
        "Dim n As Long\nFunction Answer() As Long\nAnswer=F(Tick(),(Tick()))\nEnd Function\nFunction Tick() As Long\nn=n+1\nTick=n\nEnd Function\nFunction F(ByVal a As Long,ByRef b As Long) As Long\nF=a*10+b\nEnd Function",
        Value::Long(12),
    )
}

#[test]
fn whole_array_parameters_mutate_and_resize_canonical_storage() -> R {
    answer(
        "Function Answer() As Long\nDim a() As Long\nReDim a(1 To 2)\na(1)=10\nCall Grow(a)\nAnswer=UBound(a)+a(1)+a(3)\nEnd Function\nSub Grow(ByRef values() As Long)\nReDim Preserve values(1 To 3)\nvalues(1)=values(1)+1\nvalues(3)=20\nEnd Sub",
        Value::Long(34),
    )?;
    number(
        "Function Answer() As Long\nDim a() As Long,b() As Integer\nReDim a(2)\nReDim b(2)\na=b\nEnd Function",
        13,
    )
}
#[test]
fn paramarray_is_zero_based_empty_or_packed_and_accepts_eager_expressions() -> R {
    answer(
        "Option Base 1\nFunction Answer() As String\nAnswer=Probe() & \"|\" & Probe(1,2,3)\nEnd Function\nFunction Probe(ParamArray values() As Variant) As String\nProbe=LBound(values) & \":\" & UBound(values)\nEnd Function",
        Value::String(VbaString::from_utf8("0:-1|0:2")?),
    )?;
    answer(
        "Function Answer() As Long\nAnswer=Probe(10,2+3,4)\nEnd Function\nFunction Probe(ByVal first As Long,ParamArray rest() As Variant) As Long\nProbe=first+rest(0)+rest(1)\nEnd Function",
        Value::Long(19),
    )
}
#[test]
fn option_base_controls_implicit_bounds_but_not_explicit_bounds() -> R {
    answer(
        "Option Base 1\nFunction Answer() As String\nAnswer=LBound(Array(1)) & LBound(VBA.Array(1))\nEnd Function",
        Value::String(VbaString::from_utf8("10")?),
    )?;
    answer(
        "Option Base 1\nFunction Answer() As Long\nDim a(3) As Long,b() As Long\nReDim b(4)\nAnswer=LBound(a)*100+LBound(b)*10+LBound(Array(7))\nEnd Function",
        Value::Long(111),
    )?;
    answer(
        "Option Base 1\nFunction Answer() As Long\nDim a(-2 To 3) As Long\nAnswer=LBound(a)\nEnd Function",
        Value::Long(-2),
    )
}
#[test]
fn named_byref_array_arguments_capture_indices_once_in_source_order() -> R {
    answer(
        "Dim visits As Long\nFunction Answer() As Long\nDim a(0 To 1) As Long\na(0)=10\na(1)=20\nCall F(right:=a(Index()),left:=a(Index()))\nAnswer=a(0)*100+a(1)+visits\nEnd Function\nFunction Index() As Long\nIndex=visits\nvisits=visits+1\nEnd Function\nSub F(ByRef left As Long,ByRef right As Long)\nleft=left+1\nright=right+2\nEnd Sub",
        Value::Long(1223),
    )
}
#[test]
fn resizing_between_array_reference_capture_and_call_does_not_retarget_the_alias() -> R {
    number(
        "Dim a() As Long\nFunction Answer() As Long\nReDim a(2)\nCall F(value:=a(0),other:=Resize())\nEnd Function\nFunction Resize() As Long\nReDim a(3)\nResize=1\nEnd Function\nSub F(ByRef value As Long,ByVal other As Long)\nvalue=99\nEnd Sub",
        9,
    )
}
#[test]
fn malformed_array_signatures_and_mismatched_array_arguments_are_rejected() -> R {
    for text in [
        "Option Base 1\nOption Base 0\nFunction Answer() As Long\nEnd Function",
        "Function Answer() As Long\nEnd Function\nSub F(ByRef ParamArray values() As Variant)\nEnd Sub",
        "Function Answer() As Long\nEnd Function\nSub F(ByVal values() As Long)\nEnd Sub",
        "Function Answer() As Long\nEnd Function\nSub F(ParamArray values() As Long)\nEnd Sub",
        "Function Answer() As Long\nEnd Function\nSub F(ParamArray values() As Variant,ByVal other As Long)\nEnd Sub",
        "Function Answer() As Long\nDim values() As Integer\nCall F(values)\nEnd Function\nSub F(ByRef values() As Long)\nEnd Sub",
    ] {
        assert!(session(text).is_err(), "accepted {text}");
    }
    Ok(())
}

#[test]
fn dynamically_typed_array_reference_mismatch_is_a_numbered_language_error() -> R {
    number(
        "Function Answer() As Long\nDim typed(1) As Long,v As Variant\nv=typed\nCall F(v(0))\nEnd Function\nSub F(ByRef x As Variant)\nx=1\nEnd Sub",
        13,
    )
}

#[test]
fn debugger_array_edits_preserve_declared_element_kinds() -> R {
    let mut s = session(
        "Function Answer() As Long\nDim a(1) As Long\na(0)=4\nStop\nAnswer=a(0)\nEnd Function",
    )?;
    let Outcome::Paused(p) = drive(&mut s)? else {
        return Err("array edit pause".into());
    };
    let before = s.read_local(p.handle, "a")?;
    let wrong = vba_values::VbaArray::new(vba_values::ScalarKind::Integer, vec![(0, 1)], true)?;
    assert!(s.edit_local(p.handle, "a", Value::Array(wrong)).is_err());
    assert_eq!(s.read_local(p.handle, "a")?, before);
    let mut right = vba_values::VbaArray::new(vba_values::ScalarKind::Long, vec![(0, 1)], true)?;
    right.set(0, Value::Long(42))?;
    s.edit_local(p.handle, "a", Value::Array(right))?;
    s.continue_execution()?;
    assert_eq!(
        drive(&mut s)?,
        Outcome::Completed(ReturnValue::Function(Value::Long(42)))
    );
    Ok(())
}
