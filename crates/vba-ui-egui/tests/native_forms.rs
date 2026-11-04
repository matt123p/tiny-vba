#[path = "../../vba-project/tests/support/native_forms.rs"]
mod fixture;
use egui_kittest::{Harness, kittest::Queryable};
use vba_ide::{Ide, Mode, NativeFormResource, NoHost};
use vba_ui_egui::{EditorUi, Theme, apply};
type R = Result<(), Box<dyn std::error::Error>>;
struct State {
    ide: Ide,
    view: EditorUi,
    errors: Vec<String>,
}
fn draw(ui: &mut egui::Ui, state: &mut State) {
    for command in state.view.show(ui, &state.ide, Theme::light()) {
        if let Err(error) = apply(&mut state.ide, command, &mut NoHost) {
            state.errors.push(error.to_string());
        }
    }
    if let Err(error) = state.ide.tick(&mut NoHost) {
        state.errors.push(error.to_string());
    }
}
#[test]
fn native_import_reaches_project_explorer_properties_and_actual_runtime_controls() -> R {
    let mut ide = Ide::new();
    let id = ide.import_native_form_export(
        "NativeForm.frm",
        &fixture::export(),
        1252,
        &[NativeFormResource {
            name: "NativeForm.frx".into(),
            bytes: fixture::companion(),
        }],
    )?;
    let mut view = EditorUi::default();
    view.form_view = true;
    // Native Show now also runs ordinary initial-focus frames before settling.
    let mut h = Harness::builder()
        .with_max_steps(8)
        .with_size(egui::vec2(1400.0, 1000.0))
        .build_ui_state(
            draw,
            State {
                ide,
                view,
                errors: vec![],
            },
        );
    assert!(h.query_by_label("NativeForm.frm").is_some());
    assert!(h.query_by_label("Entry (TextBox)").is_some());
    h.get_by_label("Entry (TextBox)").click();
    h.run();
    assert!(h.query_by_label("Control Name").is_some());
    h.state_mut().ide.run_form(id, true, &mut NoHost)?;
    for _ in 0..100 {
        h.run();
        if h.state().ide.mode() == Mode::Waiting {
            break;
        }
    }
    assert_eq!(h.state().ide.mode(), Mode::Waiting);
    assert!(h.query_by_label("Apply").is_some());
    h.get_by_label("Apply").click();
    for _ in 0..100 {
        h.run();
        if h.state().ide.mode() == Mode::Completed {
            break;
        }
    }
    assert_eq!(
        h.state().ide.output(),
        vec!["initialized".to_string(), "initial".to_string()]
    );
    assert!(h.state().errors.is_empty(), "{:?}", h.state().errors);
    Ok(())
}
