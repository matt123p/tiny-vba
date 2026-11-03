//! Generic host continuations only; application objects and commit policy belong to forks.
use std::collections::{BTreeMap, VecDeque};
use vba_foreign::{Completion, ForeignError, HostCall, Reply};
use vba_ide::{Action, Host, Ide, Import, Mode, ObjectRef, SuspendedCall, Value};
type R = Result<(), Box<dyn std::error::Error>>;
const ROOT: ObjectRef = ObjectRef {
    host: 44,
    object: 1,
    generation: 1,
};
#[derive(Default)]
struct TestHost {
    next: u64,
    requests: VecDeque<(u64, SuspendedCall)>,
    results: BTreeMap<u64, Result<Completion, ForeignError>>,
    cancelled: Vec<u64>,
    calls: usize,
}
impl Host for TestHost {
    fn identity(&self) -> u64 {
        44
    }
    fn invoke(&mut self, call: HostCall) -> Reply {
        self.calls += 1;
        self.next += 1;
        let token = self.next;
        let request = if call.member == "Commit" {
            SuspendedCall::DocumentEvent {
                code_name: "DocumentCode".into(),
                handler: "BeforeCommit".into(),
                arguments: vec![Value::Boolean(false)],
                owner: Some(ROOT),
            }
        } else {
            let Some(arg) = call.arguments.first() else {
                return Reply::Error(ForeignError {
                    number: 449,
                    description: "name required".into(),
                });
            };
            let Value::String(name) = &arg.value else {
                return Reply::Error(ForeignError {
                    number: 13,
                    description: "name required".into(),
                });
            };
            SuspendedCall::Procedure {
                name: String::from_utf16_lossy(name.units()),
                arguments: call
                    .arguments
                    .iter()
                    .skip(1)
                    .map(|a| a.value.clone())
                    .collect(),
            }
        };
        self.requests.push_back((token, request));
        Reply::Wait(token)
    }
    fn resume(&mut self, token: u64) -> Reply {
        match self.results.remove(&token) {
            Some(Ok(result)) => Reply::Complete(result),
            Some(Err(e)) => Reply::Error(e),
            None => Reply::Wait(token),
        }
    }
    fn cancel(&mut self, token: u64) {
        self.cancelled.push(token);
        self.results.remove(&token);
        self.requests.retain(|(t, _)| *t != token);
    }
}
fn setup(source: &str) -> Result<(Ide, TestHost), Box<dyn std::error::Error>> {
    let mut ide = Ide::new();
    ide.add_module("Calls.bas", source)?;
    ide.configure_host(
        vec![Import {
            name: "bridge".into(),
            root: "Bridge".into(),
            member: None,
            kind: vba_foreign::InvokeKind::Get,
        }],
        BTreeMap::from([("Bridge".into(), ROOT)]),
        vec![],
    )?;
    let mut host = TestHost::default();
    ide.dispatch(Action::Run, &mut host)?;
    Ok((ide, host))
}
fn tick(ide: &mut Ide, host: &mut TestHost) -> R {
    if ide.mode() == Mode::Waiting
        && let Some((token, request)) = host.requests.pop_front()
        && let Err(e) = ide.start_suspended_call(token, request, host)
    {
        host.results.insert(
            token,
            Err(ForeignError {
                number: 1004,
                description: e.message,
            }),
        );
    }
    ide.tick(host)?;
    while let Some((token, result)) = ide.take_host_call_result() {
        host.results.insert(
            token,
            result.map(|mut r| {
                r.writes.clear();
                r
            }),
        );
    }
    Ok(())
}
fn finish(ide: &mut Ide, host: &mut TestHost) -> R {
    for _ in 0..2000 {
        tick(ide, host)?;
        if !matches!(ide.mode(), Mode::Running | Mode::Waiting) {
            break;
        }
    }
    assert_eq!(ide.mode(), Mode::Completed, "{:?}", ide.diagnostics);
    Ok(())
}
#[test]
fn nested_returns_defaults_globals_and_errors_share_the_waiting_project() -> R {
    let (mut ide, mut host) = setup(
        r#"Public counter As Long
Sub Main()
Dim x As Long
Dim result As Variant
x = 5
result = bridge.Run("Adder", x)
Debug.Print result
Debug.Print x
Debug.Print counter
On Error Resume Next
result = bridge.Run("FailNow")
Debug.Print Err.Number
Err.Clear
result = bridge.Run("Hidden")
Debug.Print Err.Number
End Sub
Public Function Adder(ByRef x As Long, Optional stepSize As Long = 2) As Long
counter = counter + 1
x = x + stepSize
Adder = bridge.Run("Twice", x)
End Function
Public Function Twice(ByVal x As Long) As Long
counter = counter + 1
Twice = x * 2
End Function
Public Function FailNow() As Long
Err.Raise 17
End Function
Private Sub Hidden()
Debug.Print "must not execute"
End Sub
"#,
    )?;
    finish(&mut ide, &mut host)?;
    assert_eq!(ide.output(), ["14", "5", "2", "17", "1004"]);
    assert_eq!(host.calls, 4);
    assert!(host.cancelled.is_empty());
    Ok(())
}
#[test]
fn suspended_document_event_returns_byref_outputs_before_host_commit() -> R {
    let (mut ide, mut host) =
        setup("Sub Main()\nbridge.Commit\nDebug.Print \"resumed\"\nEnd Sub\n")?;
    // Adding source makes the running session stale; configure the document before a fresh Run.
    ide.dispatch(Action::Reset, &mut host)?;
    ide.add_document_module(
        "Document.cls",
        "DocumentCode",
        r#"Attribute VB_Name = "DocumentCode"
Private Sub BeforeCommit(ByRef Cancel As Boolean)
Cancel = True
End Sub
"#,
    )?;
    ide.dispatch(Action::Run, &mut host)?;
    let mut outputs = vec![];
    for _ in 0..2000 {
        if ide.mode() == Mode::Waiting
            && let Some((token, request)) = host.requests.pop_front()
        {
            ide.start_suspended_call(token, request, &mut host)?;
        }
        ide.tick(&mut host)?;
        while let Some((token, result)) = ide.take_host_call_result() {
            let result = result.map_err(|e| e.description)?;
            outputs = result.writes.clone();
            let mut result = result;
            result.writes.clear();
            host.results.insert(token, Ok(result));
        }
        if ide.mode() == Mode::Completed {
            break;
        }
    }
    assert_eq!(outputs, vec![(0, Value::Boolean(true))]);
    assert_eq!(ide.output(), ["resumed"]);
    Ok(())
}
#[test]
fn debugger_pause_and_reset_cancel_both_wait_tokens_exactly_once() -> R {
    let (mut ide, mut host) = setup(
        "Sub Main()\nbridge.Run \"Child\"\nEnd Sub\nPublic Sub Child()\nStop\nbridge.Run \"Leaf\"\nEnd Sub\nPublic Sub Leaf()\nDebug.Print \"not run\"\nEnd Sub\n",
    )?;
    for _ in 0..100 {
        tick(&mut ide, &mut host)?;
        if ide.mode() == Mode::Break {
            break;
        }
    }
    assert_eq!(ide.mode(), Mode::Break);
    assert!(host.cancelled.is_empty());
    ide.dispatch(Action::Run, &mut host)?;
    for _ in 0..100 {
        ide.tick(&mut host)?;
        if ide.mode() == Mode::Waiting {
            break;
        }
    }
    ide.dispatch(Action::Reset, &mut host)?;
    host.cancelled.sort_unstable();
    assert_eq!(host.cancelled, vec![1, 2]);
    assert!(ide.take_host_call_result().is_none());
    assert!(ide.output().is_empty());
    Ok(())
}

#[test]
fn host_paramarray_packs_typed_tail_and_empty_tail() -> R {
    let (mut ide, mut host) = setup(
        "Sub Main()\nDebug.Print bridge.Run(\"CountArgs\", 4, \"x\", True)\nDebug.Print bridge.Run(\"CountArgs\")\nEnd Sub\nPublic Function CountArgs(ParamArray args() As Variant) As Long\nCountArgs = UBound(args) + 1\nEnd Function\n",
    )?;
    finish(&mut ide, &mut host)?;
    assert_eq!(ide.output(), ["3", "0"]);
    Ok(())
}
#[test]
fn stale_source_and_wrong_tokens_cannot_start_a_callback() -> R {
    let (mut ide, mut host) = setup(
        "Sub Main()\nbridge.Run \"Child\"\nEnd Sub\nPublic Sub Child()\nDebug.Print \"must not run\"\nEnd Sub\n",
    )?;
    ide.tick(&mut host)?;
    let (token, request) = host.requests.pop_front().ok_or("wait")?;
    assert!(
        ide.start_suspended_call(token + 1, request.clone(), &mut host)
            .is_err()
    );
    let text = ide.modules.first().ok_or("module")?.text.clone() + "\n' edited\n";
    ide.set_text(text)?;
    assert!(ide.start_suspended_call(token, request, &mut host).is_err());
    ide.dispatch(Action::Reset, &mut host)?;
    assert_eq!(host.cancelled, [token]);
    assert!(ide.output().is_empty());
    Ok(())
}
#[test]
fn nested_work_does_not_replenish_instruction_budget() -> R {
    let (mut ide, mut host) = setup(
        "Sub Main()\nbridge.Run \"Child\"\nEnd Sub\nPublic Sub Child()\nDim i As Long\nFor i = 1 To 10000\nNext i\nEnd Sub\n",
    )?;
    ide.dispatch(Action::Reset, &mut host)?;
    ide.configure_execution_limits(vba_vm::Limits {
        instructions: 100,
        slice: 4096,
        ..Default::default()
    })?;
    ide.dispatch(Action::Run, &mut host)?;
    for _ in 0..100 {
        tick(&mut ide, &mut host)?;
        if ide.mode() == Mode::Failed {
            break;
        }
    }
    assert_eq!(ide.mode(), Mode::Failed);
    assert_eq!(host.cancelled, [1]);
    assert!(ide.take_host_call_result().is_none());
    Ok(())
}
