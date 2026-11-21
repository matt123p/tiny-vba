//! A project's forms request a host root, so the IDE authorizes every foreign
//! site of the compiled program. Most sites come from each form's generated
//! support code. A project with many small forms must still run.
use vba_ide::forms::{Bounds, ControlKind, DesignEdit};
use vba_ide::{Action, Ide, Mode, NoHost};
type R<T = ()> = Result<T, Box<dyn std::error::Error>>;

#[test]
fn a_project_with_many_small_forms_runs_a_macro() -> R {
    let mut ide = Ide::new();
    // Seven forms like these already exceeded the former fixed cap of 4,096 sites.
    for f in 0..12 {
        let id = ide.add_form(&format!("Form{f}"))?;
        for top in [12.0, 42.0] {
            ide.edit_form(
                id,
                DesignEdit::Add(
                    ControlKind::Label,
                    Bounds {
                        left: 12.0,
                        top,
                        width: 120.0,
                        height: 24.0,
                    },
                ),
            )?;
        }
    }
    ide.add_module("Main.bas", "Sub Main()\nDebug.Print \"ran\"\nEnd Sub")?;
    ide.dispatch(Action::Run, &mut NoHost)?;
    for _ in 0..2000 {
        if ide.mode() != Mode::Running {
            break;
        }
        ide.tick(&mut NoHost)?;
    }
    assert_eq!(ide.mode(), Mode::Completed, "{:?}", ide.diagnostics);
    assert_eq!(ide.output(), ["ran"]);
    Ok(())
}
