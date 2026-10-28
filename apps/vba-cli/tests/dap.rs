#![cfg(feature = "dap")]
//! Real framed stdio transcript over the shipping optional CLI, with deadlines.
use std::{
    io::{self, BufReader},
    process::{Child, ChildStdin, Command, Stdio},
    sync::mpsc,
    time::{Duration, Instant},
};
use vba_dap::{Json, json, transport};
type R<T = ()> = Result<T, Box<dyn std::error::Error>>;
struct Client {
    child: Child,
    input: ChildStdin,
    rx: mpsc::Receiver<io::Result<Json>>,
    seq: i64,
    events: Vec<Json>,
}
impl Client {
    fn new() -> R<Self> {
        let mut child = Command::new(env!("CARGO_BIN_EXE_vba-cli"))
            .arg("dap")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()?;
        let input = child.stdin.take().ok_or("stdin")?;
        let output = child.stdout.take().ok_or("stdout")?;
        let (tx, rx) = mpsc::sync_channel(32);
        std::thread::Builder::new()
            .name("dap-test-reader".into())
            .spawn(move || {
                let mut reader = BufReader::new(output);
                loop {
                    match transport::read(&mut reader) {
                        Ok(Some(value)) => {
                            if tx.send(Ok(value)).is_err() {
                                break;
                            }
                        }
                        Ok(None) => break,
                        Err(e) => {
                            let _ = tx.send(Err(e));
                            break;
                        }
                    }
                }
            })?;
        Ok(Self {
            child,
            input,
            rx,
            seq: 0,
            events: Vec::new(),
        })
    }
    fn next(&self) -> R<Json> {
        Ok(self.rx.recv_timeout(Duration::from_secs(10))??)
    }
    fn request(&mut self, command: &str, args: Json) -> R<Json> {
        self.seq += 1;
        transport::write(
            &mut self.input,
            &json!({"type":"request","seq":self.seq,"command":command,"arguments":args}),
        )?;
        loop {
            let value = self.next()?;
            if value.get("type").and_then(Json::as_str) == Some("event") {
                self.events.push(value);
                continue;
            }
            assert_eq!(
                value.get("request_seq").and_then(Json::as_i64),
                Some(self.seq)
            );
            assert_eq!(value.get("success"), Some(&Json::Bool(true)), "{value}");
            return Ok(value);
        }
    }
    fn event(&mut self, name: &str) -> R<Json> {
        if let Some(index) = self
            .events
            .iter()
            .position(|event| event.get("event").and_then(Json::as_str) == Some(name))
        {
            return Ok(self.events.remove(index));
        }
        loop {
            let value = self.next()?;
            if value.get("event").and_then(Json::as_str) == Some(name) {
                return Ok(value);
            }
            self.events.push(value);
        }
    }
    fn exited(&mut self) -> R<i32> {
        let deadline = Instant::now() + Duration::from_secs(10);
        loop {
            if let Some(exit) = self.child.try_wait()? {
                return Ok(exit.code().ok_or("no exit code")?);
            }
            if Instant::now() > deadline {
                return Err("DAP process did not exit".into());
            }
            std::thread::sleep(Duration::from_millis(10));
        }
    }
}
impl Drop for Client {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}
#[test]
fn dap_stdio_launch_break_step_inspect_and_disconnect() -> R {
    let mut client = Client::new()?;
    client.request("initialize", json!({}))?;
    client.request("launch",json!({"sourceName":"Test.bas","sourceCode":"Sub Main()\nDim n As Long\nn=1\nn=n+1\nDebug.Print n\nEnd Sub\n","stopOnEntry":true}))?;
    client.event("initialized")?;
    client.request("configurationDone", json!({}))?;
    assert_eq!(
        client
            .event("stopped")?
            .pointer("/body/reason")
            .and_then(Json::as_str),
        Some("entry")
    );
    client.request("next", json!({"threadId":1}))?;
    assert_eq!(
        client
            .event("stopped")?
            .pointer("/body/reason")
            .and_then(Json::as_str),
        Some("step")
    );
    let trace = client.request("stackTrace", json!({"threadId":1}))?;
    let frame = trace
        .pointer("/body/stackFrames/0/id")
        .ok_or("frame")?
        .clone();
    let evaluated = client.request("evaluate", json!({"frameId":frame,"expression":"n"}))?;
    assert_eq!(
        evaluated.pointer("/body/result").and_then(Json::as_str),
        Some("1")
    );
    client.request("continue", json!({"threadId":1}))?;
    client.event("terminated")?;
    assert!(client.events.iter().any(|event| {
        event
            .pointer("/body/output")
            .and_then(Json::as_str)
            .is_some_and(|text| text.trim() == "2")
    }));
    client.request("disconnect", json!({}))?;
    assert_eq!(client.exited()?, 0);
    Ok(())
}
#[test]
fn dap_stdio_rejects_oversized_frames_before_allocating_payload() -> R {
    use std::io::Write;
    let mut client = Client::new()?;
    client.input.write_all(b"Content-Length: 1048577\r\n\r\n")?;
    client.input.flush()?;
    assert_eq!(client.exited()?, 2);
    Ok(())
}
