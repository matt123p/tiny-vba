//! Scriptable CLI debugger over the same headless IDE model used by the graphical panels.
use std::{
    error::Error,
    fs::File,
    io::{self, BufRead, BufReader, Read},
};
use vba_ide::{Action, Ide, Mode, NoHost, display_value};
pub fn run(args: Vec<String>) -> Result<(), Box<dyn Error>> {
    let mut args = args.into_iter();
    let path = args
        .next()
        .ok_or("usage: debug <source.bas> [--entry name] [--script commands.txt]")?;
    let mut entry = None;
    let mut script = None;
    while let Some(flag) = args.next() {
        match flag.as_str() {
            "--entry" => entry = Some(args.next().ok_or("missing entry")?),
            "--script" => script = Some(args.next().ok_or("missing script")?),
            _ => return Err("unknown debugger option".into()),
        }
    }
    let mut text = String::new();
    File::open(&path)?
        .take((vba_ide::SourceFile::MAX_BYTES + 1) as u64)
        .read_to_string(&mut text)?;
    let name = std::path::Path::new(&path)
        .file_name()
        .and_then(|p| p.to_str())
        .ok_or("invalid module filename")?;
    if !name.to_ascii_lowercase().ends_with(".bas") {
        return Err("debug requires plain .bas source".into());
    }
    let mut ide = Ide::new();
    ide.add_module(name, &text)?;
    ide.entry = entry
        .or_else(|| {
            ide.active_module()
                .and_then(|m| m.procedures.first())
                .map(|p| p.name.clone())
        })
        .ok_or("no entry procedure")?;
    let mut reader: Box<dyn BufRead> = match script {
        Some(path) => Box::new(BufReader::new(File::open(path)?)),
        None => Box::new(BufReader::new(io::stdin())),
    };
    println!(
        "DEBUGGER READY — run, break <line>, step, next, out, locals, stack, print <expr>, set <local = expr>, watch <expr>, reset, quit"
    );
    loop {
        let mut line = String::new();
        if reader.by_ref().take(4097).read_line(&mut line)? == 0 {
            break;
        }
        if line.len() > 4096 {
            return Err("debugger command size limit".into());
        }
        let line = line.trim();
        let (command, arg) = line.split_once(' ').unwrap_or((line, ""));
        match command {
            "" => continue,
            "quit" | "q" => {
                if ide.enabled(Action::Reset) {
                    ide.dispatch(Action::Reset, &mut NoHost)?;
                }
                break;
            }
            "run" | "continue" => ide.dispatch(Action::Run, &mut NoHost)?,
            "step" => ide.dispatch(Action::StepInto, &mut NoHost)?,
            "next" => ide.dispatch(Action::StepOver, &mut NoHost)?,
            "out" => ide.dispatch(Action::StepOut, &mut NoHost)?,
            "pause" => ide.dispatch(Action::Pause, &mut NoHost)?,
            "reset" => ide.dispatch(Action::Reset, &mut NoHost)?,
            "break" => {
                let line = arg.parse::<usize>()?;
                let m = ide.active_module_mut().ok_or("no module")?;
                m.caret = m.line_start(line);
                ide.dispatch(Action::ToggleBreakpoint, &mut NoHost)?;
            }
            "locals" => {
                for (name, kind, value) in ide.locals() {
                    println!("{name}: {kind:?} = {}", display_value(&value));
                }
            }
            "stack" => {
                for frame in ide.frames() {
                    println!("FRAME {} {:?}", frame.id, frame.span);
                }
            }
            "print" => println!("VALUE {}", display_value(&ide.evaluate(arg)?)),
            "set" => ide.immediate(arg)?,
            "watch" => ide.add_watch(arg)?,
            _ => return Err("unknown debugger command".into()),
        }
        for _ in 0..10000 {
            if !matches!(ide.mode(), Mode::Running | Mode::Waiting) {
                break;
            }
            ide.tick(&mut NoHost)?;
        }
        println!("STATE {}", ide.mode().label());
        if let Some(stop) = ide.execution() {
            println!("AT {}:{}", stop.procedure, stop.line);
        }
        for line in ide.output() {
            println!("OUTPUT {line}");
        }
        if ide.mode() == Mode::Failed {
            return Err(ide
                .diagnostics
                .first()
                .map_or("execution failed".into(), ToString::to_string)
                .into());
        }
    }
    Ok(())
}
