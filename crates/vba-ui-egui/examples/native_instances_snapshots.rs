//! Original multi-instance UserForm renders using the production import/VM/UI path.
#[path = "../../vba-project/tests/support/native_forms.rs"]
mod fixture;
use egui_kittest::Harness;
use vba_ide::{Action, Ide, Mode, NativeFormResource, NoHost};
use vba_ui_egui::{EditorUi, Theme};
struct State {
    ide: Ide,
    view: EditorUi,
    theme: Theme,
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let folder = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "reports/ui".into());
    std::fs::create_dir_all(&folder)?;
    let lists = std::env::args()
        .nth(2)
        .is_some_and(|option| option == "--list-controls");
    for (name, theme, width, scale) in [
        ("light", Theme::light(), 1400., 1.),
        ("dark", Theme::dark(), 1400., 1.),
        ("narrow", Theme::light(), 700., 1.),
        ("scaled", Theme::light(), 1200., 1.5),
        ("host", Theme::host_example(), 1400., 1.),
    ] {
        let mut ide = Ide::new();
        ide.import_native_form_export(
            "NativeForm.frm",
            &fixture::export(),
            1252,
            &[NativeFormResource {
                name: "NativeForm.frx".into(),
                bytes: if lists {
                    fixture::list_companion()
                } else {
                    fixture::boolean_companion()
                },
            }],
        )?;
        let main = if lists {
            "Sub Main()\nDim a As New NativeForm,b As New NativeForm\na.Caption=\"First window\"\nb.Caption=\"Second window\"\na.List1.AddItem \"alpha\"\na.List1.AddItem \"beta\"\na.List1.List(0,1)=\"A\"\na.List1.List(1,1)=\"B\"\na.List1.ListIndex=1\nb.List1.AddItem \"separate instance\"\nb.List1.List(0,1)=\"C\"\nb.List1.ListIndex=0\na.Show 0\nb.Show 0\nEnd Sub"
        } else {
            "Sub Main()\nDim a As NativeForm,b As NativeForm\nSet a=New NativeForm\nSet b=New NativeForm\na.Caption=\"First window\"\nb.Caption=\"Second window\"\na.Entry.Text=\"Value one\"\nb.Entry.Text=\"Value two\"\na.Show 0\nb.Show 0\nEnd Sub"
        };
        ide.add_module("Main.bas", main)?;
        ide.dispatch(Action::Run, &mut NoHost)?;
        for _ in 0..100 {
            ide.tick(&mut NoHost)?;
            if ide.mode() == Mode::Completed {
                break;
            }
        }
        if ide.form_instances()?.len() != 2 {
            return Err("expected two live native form instances".into());
        }
        let mut h = Harness::builder()
            .with_size(egui::vec2(width, 1000.))
            .with_pixels_per_point(scale)
            .wgpu()
            .build_ui_state(
                |ui, s: &mut State| {
                    let _ = s.view.show(ui, &s.ide, s.theme);
                },
                State {
                    ide,
                    view: EditorUi::default(),
                    theme,
                },
            );
        h.run_steps(5);
        let prefix = if lists {
            "m7a4-list-instances"
        } else {
            "m7a4-instances"
        };
        h.render()?.save(format!("{folder}/{prefix}-{name}.png"))?;
    }
    Ok(())
}
