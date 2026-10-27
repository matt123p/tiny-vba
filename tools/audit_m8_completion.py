#!/usr/bin/env python3
"""Reproduce open M8 audit defects using temporary Rust integration tests.

Maintenance-only orchestration; all product behavior executes the actual Rust model/UI.
Exit 0 means every audit probe passes. Exit 1 means at least one defect remains.
Temporary probe files are removed even when cargo reports a failure.
"""
from pathlib import Path
import subprocess
import tempfile

ROOT = Path(__file__).resolve().parents[1]

UI_PROBES = r'''
#[test]
fn audit_find_next_advances_past_current_match() -> R {
    let mut h = Harness::builder().with_size(egui::vec2(1200.0,800.0))
        .build_ui_state(draw, Fixture::new()?);
    h.key_press_modifiers(egui::Modifiers::CTRL,egui::Key::F); h.run();
    h.event(egui::Event::Text("counter".into())); h.run();
    h.get_by_label("Find Next").click(); h.run();
    let first = h.state().ide.active_module().ok_or("source")?.caret;
    h.get_by_label("Find Next").click(); h.run();
    let second = h.state().ide.active_module().ok_or("source")?.caret;
    assert!(second > first, "Find Next must advance: first={first}, second={second}");
    Ok(())
}
'''
MODEL_PROBES = r'''
#[test]
fn audit_lone_cr_source_can_set_a_statement_breakpoint() -> R {
    let mut ide = Ide::new();
    ide.add_module("LoneCr.bas", "Sub Main()\rDim x As Long\rx = 1\rEnd Sub\r")?;
    ide.dispatch(Action::Compile, &mut NoHost)?;
    let byte = ide.active_module().and_then(|m| m.text.find("x = 1"))
        .ok_or("statement")?;
    let result = ide.toggle_breakpoint_at(byte);
    assert!(result.is_ok(),
        "valid lone-CR source must use the compiler's line coordinates: {result:?}");
    Ok(())
}
#[test]
fn audit_static_state_survives_run_until_explicit_reset() -> R {
    let mut ide = Ide::new();
    ide.add_module("Static.bas", "Sub Main()\nStatic counter As Long\ncounter = counter + 1\nDebug.Print counter\nEnd Sub\n")?;
    ide.dispatch(Action::Run, &mut NoHost)?; tick(&mut ide, &mut NoHost)?;
    assert_eq!(ide.output(), vec!["1".to_string()]);
    ide.dispatch(Action::Run, &mut NoHost)?; tick(&mut ide, &mut NoHost)?;
    assert_eq!(ide.output(), vec!["2".to_string()],
        "normal completion must preserve project static state until Reset");
    Ok(())
}
'''


def run():
    failed = False
    for package, base, probes in [
        ('vba-ui-egui', 'crates/vba-ui-egui/tests/interaction.rs', UI_PROBES),
        ('vba-ide', 'crates/vba-ide/tests/workflows.rs', MODEL_PROBES),
    ]:
        existing = ROOT / base
        fixture = '\n'.join(line for line in existing.read_text().splitlines()
                            if not line.startswith('//!'))
        with tempfile.NamedTemporaryFile(mode='w', prefix='m8_completion_audit_',
                                         suffix='.rs', dir=existing.parent,
                                         delete=False) as file:
            target = Path(file.name)
            file.write(fixture + '\n' + probes)
        try:
            result = subprocess.run(
                ['cargo', 'test', '-p', package, '--test', target.stem,
                 'audit_', '--locked', '--', '--nocapture'], cwd=ROOT)
            failed |= result.returncode != 0
        finally:
            target.unlink(missing_ok=True)
    print('M8_COMPLETION_AUDIT: ' + ('FAIL — open defects remain' if failed else 'PASS'))
    return int(failed)


if __name__ == '__main__':
    raise SystemExit(run())
