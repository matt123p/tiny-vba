//! Independently authored DAP client transcript and framing regressions.
use std::io::Cursor;
use vba_dap::{Adapter, Json, json, transport};
use vba_ide::{Ide, Mode, NoHost};
type R = Result<(), Box<dyn std::error::Error>>;
fn send(adapter: &mut Adapter, command: &str, args: Json) -> Json {
    adapter
        .request(
            &json!({"seq":1,"type":"request","command":command,"arguments":args}),
            &mut NoHost,
        )
        .into_iter()
        .next()
        .unwrap_or(Json::Null)
}
fn success(response: &Json) -> &Json {
    assert_eq!(
        response.get("success"),
        Some(&Json::Bool(true)),
        "{response}"
    );
    response.get("body").unwrap_or(&Json::Null)
}
fn failed(response: &Json) {
    assert_eq!(
        response.get("success"),
        Some(&Json::Bool(false)),
        "{response}"
    );
}
fn drain(adapter: &mut Adapter) -> Vec<Json> {
    let mut events = Vec::new();
    for _ in 0..1000 {
        events.extend(adapter.tick(&mut NoHost));
        if !matches!(adapter.ide.mode(), Mode::Running | Mode::Waiting) {
            return events;
        }
    }
    assert!(
        !matches!(adapter.ide.mode(), Mode::Running | Mode::Waiting),
        "timeout"
    );
    events
}
#[test]
fn initialize_break_inspect_edit_step_resume_and_disconnect() -> R {
    let mut adapter = Adapter::new(Ide::new());
    failed(&send(&mut adapter, "launch", json!({})));
    let initialized = send(&mut adapter, "initialize", json!({}));
    assert_eq!(
        success(&initialized).get("supportsSetVariable"),
        Some(&Json::Bool(true))
    );
    failed(&send(&mut adapter, "initialize", json!({})));
    success(&send(
        &mut adapter,
        "launch",
        json!({"sourceCode":"Sub Main()\nDim x As Long\nx=1\nx=x+2\nDebug.Print x\nEnd Sub\n","sourceName":"Test.bas"}),
    ));
    let set = send(
        &mut adapter,
        "setBreakpoints",
        json!({"source":{"path":"Test.bas"},"breakpoints":[{"line":4}]}),
    );
    assert_eq!(
        success(&set).pointer("/breakpoints/0/verified"),
        Some(&Json::Bool(true))
    );
    success(&send(&mut adapter, "configurationDone", json!({})));
    let events = drain(&mut adapter);
    assert!(
        events
            .iter()
            .any(|e| e.get("event").and_then(Json::as_str) == Some("stopped"))
    );
    let stack = send(&mut adapter, "stackTrace", json!({"threadId":1}));
    let frame = success(&stack)
        .pointer("/stackFrames/0/id")
        .ok_or("frame")?
        .clone();
    let scopes = send(&mut adapter, "scopes", json!({"frameId":frame}));
    let scope = success(&scopes)
        .pointer("/scopes/0/variablesReference")
        .ok_or("scope")?
        .clone();
    let vars = send(
        &mut adapter,
        "variables",
        json!({"variablesReference":scope}),
    );
    assert_eq!(
        success(&vars)
            .pointer("/variables/0/value")
            .and_then(Json::as_str),
        Some("1")
    );
    success(&send(
        &mut adapter,
        "setVariable",
        json!({"variablesReference":scope,"name":"x","value":"10"}),
    ));
    failed(&send(
        &mut adapter,
        "evaluate",
        json!({"frameId":frame,"expression":"Dangerous()"}),
    ));
    success(&send(&mut adapter, "next", json!({"threadId":1})));
    let stepped = drain(&mut adapter);
    assert!(
        stepped.iter().any(
            |event| event.get("event").and_then(Json::as_str) == Some("stopped")
                && event.pointer("/body/reason").and_then(Json::as_str) == Some("step")
        )
    );
    failed(&send(
        &mut adapter,
        "variables",
        json!({"variablesReference":scope}),
    ));
    let value = send(&mut adapter, "evaluate", json!({"expression":"x"}));
    assert_eq!(
        success(&value).get("result").and_then(Json::as_str),
        Some("12")
    );
    success(&send(&mut adapter, "continue", json!({"threadId":1})));
    let events = drain(&mut adapter);
    assert!(
        events
            .iter()
            .any(|e| e.get("event").and_then(Json::as_str) == Some("terminated"))
    );
    assert!(adapter.ide.output().iter().any(|s| s.trim() == "12"));
    success(&send(&mut adapter, "disconnect", json!({})));
    assert!(adapter.disconnected());
    failed(&send(&mut adapter, "continue", json!({"threadId":1})));
    Ok(())
}
#[test]
fn framing_is_utf8_byte_counted_bounded_and_rejects_bad_input() -> R {
    let value =
        json!({"seq":1,"type":"request","command":"evaluate","arguments":{"expression":"\"λ\""}});
    let mut wire = Vec::new();
    transport::write(&mut wire, &value)?;
    assert_eq!(transport::read(&mut Cursor::new(wire))?, Some(value));
    for wire in [
        b"Content-Length: 1048577\r\n\r\n".as_slice(),
        b"Content-Length: 1\n\nX",
        b"Content-Length: 2\r\nContent-Length: 2\r\n\r\n{}",
        b"Content-Length: 2\r\n\r\n\xff\xff",
        b"Content-Length: 2\r\n\r\n{",
    ] {
        assert!(transport::read(&mut Cursor::new(wire)).is_err());
    }
    assert_eq!(transport::read(&mut Cursor::new(Vec::<u8>::new()))?, None);
    Ok(())
}
#[test]
fn running_loop_can_pause_and_disconnect_without_restarting() -> R {
    let mut adapter = Adapter::new(Ide::new());
    success(&send(&mut adapter, "initialize", json!({})));
    success(&send(
        &mut adapter,
        "launch",
        json!({"sourceCode":"Sub Main()\nDim x As Long\nFor x=1 To 1000000\nNext x\nEnd Sub\n"}),
    ));
    success(&send(&mut adapter, "configurationDone", json!({})));
    adapter.tick(&mut NoHost);
    success(&send(&mut adapter, "pause", json!({"threadId":1})));
    drain(&mut adapter);
    assert_eq!(adapter.ide.mode(), Mode::Break);
    success(&send(&mut adapter, "disconnect", json!({})));
    assert_eq!(adapter.ide.mode(), Mode::Design);
    Ok(())
}

#[test]
fn stop_on_entry_and_unsupported_breakpoint_options_are_honest() -> R {
    let mut adapter = Adapter::new(Ide::new());
    success(&send(&mut adapter, "initialize", json!({})));
    success(&send(
        &mut adapter,
        "launch",
        json!({"sourceCode":"Sub Main()\nDebug.Print 1\nEnd Sub\n","stopOnEntry":true}),
    ));
    let requested = send(
        &mut adapter,
        "setBreakpoints",
        json!({"source":{"path":"Module1.bas"},"breakpoints":[{"line":2,"condition":"False"}]}),
    );
    assert_eq!(
        success(&requested).pointer("/breakpoints/0/verified"),
        Some(&Json::Bool(false))
    );
    success(&send(&mut adapter, "configurationDone", json!({})));
    let events = drain(&mut adapter);
    assert!(
        events
            .iter()
            .any(|event| event.pointer("/body/reason").and_then(Json::as_str) == Some("entry"))
    );
    assert!(adapter.ide.output().is_empty());
    success(&send(&mut adapter, "disconnect", json!({})));
    Ok(())
}
