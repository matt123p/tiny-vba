//! Generic embedding proof: the host owns object identity, side effects and wait-token cleanup.
use std::{collections::BTreeMap, error::Error};
use vba_foreign::{Completion, HostCall, InvokeKind, Reply};
use vba_ide::{Action, ApiMember, Host, Ide, Import, Mode, ObjectRef, Value};
type R = Result<(), Box<dyn Error>>;
const ROOT: ObjectRef = ObjectRef {
    host: 42,
    object: 7,
    generation: 1,
};
struct DocumentHost {
    writes: usize,
    polls: usize,
    cancelled: Vec<u64>,
    wait: bool,
}
impl Host for DocumentHost {
    fn identity(&self) -> u64 {
        ROOT.host
    }
    fn invoke(&mut self, _call: HostCall) -> Reply {
        self.writes += 1;
        if self.wait {
            Reply::Wait(71)
        } else {
            complete()
        }
    }
    fn resume(&mut self, _token: u64) -> Reply {
        self.polls += 1;
        complete()
    }
    fn cancel(&mut self, token: u64) {
        self.cancelled.push(token);
    }
}
fn complete() -> Reply {
    Reply::Complete(Completion {
        value: Value::Empty,
        writes: vec![],
    })
}
fn setup(wait: bool) -> Result<(Ide, DocumentHost), Box<dyn Error>> {
    let mut ide = Ide::new();
    ide.add_module("Bridge.bas","Sub Main()\nDim counter As Long\ncounter = 1\nbridge.Record counter\ncounter = counter + 1\nDebug.Print counter\nEnd Sub\n")?;
    ide.configure_host(
        vec![Import {
            name: "bridge".into(),
            root: "Document".into(),
            member: Some("Record".into()),
            kind: InvokeKind::Call,
        }],
        BTreeMap::from([("Document".into(), ROOT)]),
        vec![ApiMember {
            library: "AuthoredTest".into(),
            owner: "bridge".into(),
            name: "Record".into(),
            signature: "Sub Record(ByVal value As Long)".into(),
            supported: true,
        }],
    )?;
    Ok((
        ide,
        DocumentHost {
            writes: 0,
            polls: 0,
            cancelled: vec![],
            wait,
        },
    ))
}
#[test]
fn embedded_break_edit_resume_dispatches_host_effect_exactly_once() -> R {
    let (mut ide, mut host) = setup(false)?;
    let byte = ide
        .active_module()
        .and_then(|m| m.text.find("counter = counter +"))
        .ok_or("statement")?;
    ide.toggle_breakpoint_at(byte)?;
    ide.dispatch(Action::Run, &mut host)?;
    for _ in 0..100 {
        ide.tick(&mut host)?;
        if ide.mode() == Mode::Break {
            break;
        }
    }
    assert_eq!(ide.mode(), Mode::Break);
    assert_eq!(host.writes, 1);
    ide.edit_local("counter", "40")?;
    ide.dispatch(Action::Run, &mut host)?;
    for _ in 0..100 {
        ide.tick(&mut host)?;
        if ide.mode() == Mode::Completed {
            break;
        }
    }
    assert_eq!(ide.mode(), Mode::Completed);
    assert_eq!(host.writes, 1);
    assert!(ide.output().iter().any(|s| s.contains("41")));
    assert_eq!(ide.completions("bridge.Re"), vec!["Record".to_string()]);
    Ok(())
}
#[test]
fn reset_cancels_wait_without_undoing_host_effect() -> R {
    let (mut ide, mut host) = setup(true)?;
    ide.dispatch(Action::Run, &mut host)?;
    ide.tick(&mut host)?;
    assert_eq!(ide.mode(), Mode::Waiting);
    ide.dispatch(Action::Reset, &mut host)?;
    assert_eq!(host.writes, 1);
    assert_eq!(host.polls, 0);
    assert_eq!(host.cancelled, vec![71]);
    assert_eq!(ide.mode(), Mode::Design);
    Ok(())
}
#[test]
fn successful_host_wait_resumes_exactly_once_in_bounded_ticks() -> R {
    let (mut ide, mut host) = setup(true)?;
    ide.dispatch(Action::Run, &mut host)?;
    for _ in 0..100 {
        ide.tick(&mut host)?;
        if ide.mode() == Mode::Completed {
            break;
        }
    }
    assert_eq!(ide.mode(), Mode::Completed);
    assert_eq!(host.writes, 1);
    assert_eq!(host.polls, 1);
    Ok(())
}
