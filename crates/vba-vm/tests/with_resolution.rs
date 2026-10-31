//! M6 resolution acceptance: `With` receivers, document-module implicit `Me`
//! members, and project declarations shadowing host globals — proven against
//! the real compiler and VM with a scripted generic host double.
use std::{collections::BTreeMap, collections::VecDeque, error::Error};
use vba_compiler::{CompileOptions, compile_modules};
use vba_diagnostics::{SourceFile, SourceId};
use vba_foreign::{Completion, Host, HostCall, Import, InvokeKind, Reply};
use vba_syntax::{ModuleKind, parse};
use vba_testkit::Event;
use vba_values::{ObjectRef, Value};
use vba_vm::{HostBinding, Limits, Outcome, ReturnValue, Session};
type R<T = ()> = Result<T, Box<dyn Error>>;

const ROOT: ObjectRef = ObjectRef {
    host: 45,
    object: 1,
    generation: 1,
};
const CHILD: ObjectRef = ObjectRef {
    host: 45,
    object: 2,
    generation: 1,
};
const CELL: ObjectRef = ObjectRef {
    host: 45,
    object: 4,
    generation: 1,
};
const OWNER: ObjectRef = ObjectRef {
    host: 45,
    object: 3,
    generation: 1,
};

/// Scripted double accepting several live object targets, mirroring how a real
/// host serves dispatches across the objects it handed out.
struct MultiHost {
    identity: u64,
    targets: Vec<ObjectRef>,
    replies: VecDeque<Reply>,
    trace: Vec<Event>,
}
impl MultiHost {
    fn new(targets: Vec<ObjectRef>, replies: Vec<Reply>) -> Self {
        Self {
            identity: 45,
            targets,
            replies: replies.into(),
            trace: Vec::new(),
        }
    }
    fn next(&mut self, event: Event) -> Reply {
        self.trace.push(event);
        self.replies.pop_front().unwrap_or_else(|| {
            Reply::Error(vba_foreign::ForeignError {
                number: 5,
                description: "host script exhausted".into(),
            })
        })
    }
}
impl Host for MultiHost {
    fn identity(&self) -> u64 {
        self.identity
    }
    fn invoke(&mut self, call: HostCall) -> Reply {
        if !self.targets.contains(&call.target) {
            return Reply::Error(vba_foreign::ForeignError {
                number: 91,
                description: "stale or foreign object reference".into(),
            });
        }
        self.next(Event::Invoke(call))
    }
    fn resume(&mut self, token: u64) -> Reply {
        self.next(Event::Resume(token))
    }
    fn cancel(&mut self, token: u64) {
        self.trace.push(Event::Cancel(token));
    }
}

fn complete(value: Value) -> Reply {
    Reply::Complete(Completion {
        value,
        writes: vec![],
    })
}

fn compile(
    modules: &[(SourceId, &str, &str, ModuleKind)],
    entry: &str,
    imports: &[Import],
    document_members: &BTreeMap<String, Vec<String>>,
) -> R<vba_ir::VerifiedProgram> {
    let mut parsed = Vec::new();
    for (id, name, text, kind) in modules.iter() {
        parsed.push(parse(
            &SourceFile::new(*id, (*name).into(), (*text).into())?,
            *kind,
        )?);
    }
    Ok(compile_modules(
        &parsed,
        &CompileOptions {
            entry: entry.into(),
            imports: imports.to_vec(),
            capabilities: vba_vm::capabilities(),
            document_members: document_members.clone(),
            ..Default::default()
        },
    )?)
}

fn drive(s: &mut Session, host: &mut MultiHost) -> R<Outcome> {
    for _ in 0..1000 {
        let outcome = s.run_slice(host, 64, &mut |_| Ok(false))?;
        if outcome != Outcome::Yielded {
            return Ok(outcome);
        }
    }
    Err("test budget".into())
}

fn invokes(host: &MultiHost) -> Vec<String> {
    host.trace
        .iter()
        .filter_map(|event| match event {
            Event::Invoke(call) => Some(format!(
                "{:?} {:?} {} args={}",
                call.target,
                call.kind,
                call.member,
                call.arguments.len()
            )),
            _ => None,
        })
        .collect()
}

fn roots() -> BTreeMap<String, ObjectRef> {
    BTreeMap::from([("workbook".to_string(), ROOT)])
}

#[test]
fn with_receivers_dispatch_members_and_restore_the_outer_block() -> R {
    let main = "Function Answer() As Long\nWith Sheet\n.Value=5\nWith .Child\n.Value=9\nEnd With\nAnswer=.Value\nEnd With\nEnd Function";
    let program = compile(
        &[(SourceId(1), "Main.bas", main, ModuleKind::Standard)],
        "Answer",
        &[Import {
            name: "Sheet".into(),
            root: "Workbook".into(),
            member: None,
            kind: InvokeKind::Get,
        }],
        &BTreeMap::new(),
    )?;
    let binding = HostBinding::for_program(&program, ROOT.host, roots())?;
    let mut session = Session::new(program, 1, vec![], Some(binding), Limits::default())?;
    let mut host = MultiHost::new(
        vec![ROOT, CHILD],
        vec![
            complete(Value::Empty),
            complete(Value::Object(CHILD)),
            complete(Value::Empty),
            complete(Value::Long(5)),
        ],
    );
    assert_eq!(
        drive(&mut session, &mut host)?,
        Outcome::Completed(ReturnValue::Function(Value::Long(5)))
    );
    assert_eq!(
        invokes(&host),
        vec![
            format!("{ROOT:?} Let Value args=1"),
            format!("{ROOT:?} Get Child args=0"),
            format!("{CHILD:?} Let Value args=1"),
            format!("{ROOT:?} Get Value args=0"),
        ],
        "leading dots hit the innermost receiver, then the outer block is restored"
    );
    assert!(host.replies.is_empty());
    Ok(())
}

const SHEET_SOURCE: &str = "Attribute VB_Name = \"SheetCode\"\nPublic Sub Changed()\nRange(\"A1\").Value=5\nWith Range(\"B2\")\n.Value=7\nEnd With\nEnd Sub";

#[test]
fn document_implicit_me_members_dispatch_on_the_live_owner_object() -> R {
    let mut members = BTreeMap::new();
    members.insert("sheetcode".to_string(), vec!["Range".to_string()]);
    let program = compile(
        &[
            (
                SourceId(1),
                "Main.bas",
                "Sub Main()\nEnd Sub",
                ModuleKind::Standard,
            ),
            (SourceId(2), "Sheet.cls", SHEET_SOURCE, ModuleKind::Document),
        ],
        "Main",
        &[],
        &members,
    )?;
    let binding = HostBinding::for_program(&program, OWNER.host, BTreeMap::new())?;
    let mut session = Session::new(program, 1, vec![], Some(binding), Limits::default())?;
    assert_eq!(
        drive(&mut session, &mut MultiHost::new(vec![OWNER], vec![]))?,
        Outcome::Completed(ReturnValue::Sub)
    );
    session.schedule_document_callback_with_owner("SheetCode", "Changed", vec![], Some(OWNER))?;
    let mut host = MultiHost::new(
        vec![OWNER, CELL],
        vec![
            complete(Value::Object(CELL)),
            complete(Value::Empty),
            complete(Value::Object(CELL)),
            complete(Value::Empty),
        ],
    );
    for _ in 0..session.pending_callbacks() {
        assert_eq!(
            session.run_next_callback(&mut host, 64, &mut |_| Ok(false))?,
            Outcome::Completed(ReturnValue::Sub)
        );
    }
    assert_eq!(
        invokes(&host),
        vec![
            format!("{OWNER:?} Call Range args=1"),
            format!("{CELL:?} Let Value args=1"),
            format!("{OWNER:?} Call Range args=1"),
            format!("{CELL:?} Let Value args=1"),
        ],
        "bare Range resolves to the bound worksheet owner, including With receivers"
    );
    assert!(host.replies.is_empty());
    Ok(())
}

#[test]
fn without_a_declaration_bare_document_names_are_not_silently_global() -> R {
    let result = compile(
        &[
            (
                SourceId(1),
                "Main.bas",
                "Sub Main()\nEnd Sub",
                ModuleKind::Standard,
            ),
            (SourceId(2), "Sheet.cls", SHEET_SOURCE, ModuleKind::Document),
        ],
        "Main",
        &[],
        &BTreeMap::new(),
    );
    assert!(
        result.is_err(),
        "Range is neither declared nor an implicit member here"
    );
    Ok(())
}

#[test]
fn project_declarations_shadow_host_globals_for_calls_and_values() -> R {
    let main = "Function Answer() As Long\nAnswer=Helper\nEnd Function\nFunction Helper() As Long\nHelper=42\nEnd Function";
    let program = compile(
        &[(SourceId(1), "Main.bas", main, ModuleKind::Standard)],
        "Answer",
        &[Import {
            name: "Helper".into(),
            root: "Workbook".into(),
            member: Some("Helper".into()),
            kind: InvokeKind::Get,
        }],
        &BTreeMap::new(),
    )?;
    let binding = HostBinding::for_program(&program, ROOT.host, roots())?;
    let mut session = Session::new(program, 1, vec![], Some(binding), Limits::default())?;
    let mut host = MultiHost::new(vec![ROOT], vec![]);
    assert_eq!(
        drive(&mut session, &mut host)?,
        Outcome::Completed(ReturnValue::Function(Value::Long(42)))
    );
    assert!(
        host.trace.is_empty(),
        "the project Function wins without any host dispatch"
    );
    Ok(())
}
