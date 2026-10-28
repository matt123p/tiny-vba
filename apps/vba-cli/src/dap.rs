//! Explicit optional stdio shell. Reading input never blocks the debugger pump.
use std::{io::Read, sync::mpsc, time::Duration};
use vba_dap::{Adapter, transport};
use vba_ide::{Ide, NoHost, SourceFile};
pub fn serve(path: Option<&str>) -> Result<(), Box<dyn std::error::Error>> {
    let mut ide = Ide::new();
    if let Some(path) = path {
        let mut text = String::new();
        std::fs::File::open(path)?
            .take((SourceFile::MAX_BYTES + 1) as u64)
            .read_to_string(&mut text)?;
        if text.len() > SourceFile::MAX_BYTES {
            return Err("source budget exceeded".into());
        }
        let name = std::path::Path::new(path)
            .file_name()
            .and_then(|p| p.to_str())
            .ok_or("source name required")?;
        ide.add_module(name, &text)?;
    }
    let mut adapter = Adapter::new(ide);
    let (sender, receiver) = mpsc::sync_channel(16);
    // Input thread owns stdin until EOF/process exit; it never sees IDE/host state.
    std::thread::Builder::new()
        .name("vba-dap-input".into())
        .spawn(move || {
            let input = std::io::stdin();
            let mut reader = input.lock();
            loop {
                let result = transport::read(&mut reader).map_err(|e| e.to_string());
                let done = !matches!(&result, Ok(Some(_)));
                if sender.send(result).is_err() || done {
                    break;
                }
            }
        })?;
    let stdout = std::io::stdout();
    let mut writer = stdout.lock();
    while !adapter.disconnected() {
        match receiver.recv_timeout(Duration::from_millis(5)) {
            Ok(Ok(Some(request))) => {
                for message in adapter.request(&request, &mut NoHost) {
                    transport::write(&mut writer, &message)?;
                }
            }
            Ok(Ok(None)) | Err(mpsc::RecvTimeoutError::Disconnected) => {
                adapter.ide.close(&mut NoHost, true)?;
                break;
            }
            Ok(Err(message)) => {
                adapter.ide.close(&mut NoHost, true)?;
                return Err(message.into());
            }
            Err(mpsc::RecvTimeoutError::Timeout) => {}
        }
        for message in adapter.tick(&mut NoHost) {
            transport::write(&mut writer, &message)?;
        }
    }
    Ok(())
}
