//! Original multi-instance UserForm renders using the production import/VM/UI path.
#[path = "../../vba-project/tests/support/native_forms.rs"]
mod fixture;
use egui_kittest::{Harness, kittest::Queryable};
use vba_ide::{Action, Ide, Mode, NativeFormResource, NoHost};
use vba_ui_egui::{EditorUi, Theme};
struct State {
    ide: Ide,
    view: EditorUi,
    theme: Theme,
    metadata: bool,
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let folder = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "reports/ui".into());
    std::fs::create_dir_all(&folder)?;
    let images = std::env::args()
        .nth(2)
        .is_some_and(|o| o == "--image-controls");
    let pages = std::env::args()
        .nth(2)
        .is_some_and(|o| o == "--page-controls");
    let tabs = std::env::args()
        .nth(2)
        .is_some_and(|o| o == "--tab-controls");
    let combos = std::env::args()
        .nth(2)
        .is_some_and(|option| option == "--combo-controls");
    let lists = std::env::args()
        .nth(2)
        .is_some_and(|option| option == "--list-controls");
    let frames = std::env::args()
        .nth(2)
        .is_some_and(|o| o == "--frame-controls");
    let metadata = std::env::args().nth(2).is_some_and(|o| o == "--metadata");
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
                bytes: if images {
                    fixture::image_companion()
                } else if pages {
                    fixture::multipage_companion(1)
                } else if tabs {
                    fixture::tab_companion(1)
                } else if frames {
                    fixture::frame_companion(2)
                } else if metadata {
                    fixture::metadata_companion()
                } else if combos {
                    fixture::combo_companion(3, 1)
                } else if lists {
                    fixture::list_companion()
                } else {
                    fixture::boolean_companion()
                },
            }],
        )?;
        let main = if images {
            "Sub Main()\nSet NativeForm.Image2.Picture=NativeForm.Image1.Picture\nNativeForm.Image2.PictureTiling=True\nNativeForm.Image2.PictureSizeMode=fmPictureSizeModeClip\nNativeForm.Show 0\nEnd Sub"
        } else if pages {
            "Sub Main()\nNativeForm.Show 0\nNativeForm.Multi1.SetFocus\nEnd Sub"
        } else if tabs {
            "Sub Main()\nNativeForm.Show 0\nNativeForm.Tabs1.SetFocus\nEnd Sub"
        } else if frames {
            "Sub Main()\nNativeForm.Show 0\nEnd Sub"
        } else if metadata {
            "Sub Main()\nNativeForm.Entry.ControlTipText=\"A native form control tip\"\nNativeForm.Show 0\nEnd Sub"
        } else if lists || combos {
            "Sub Main()\nDim a As New NativeForm,b As New NativeForm\na.Caption=\"First window\"\nb.Caption=\"Second window\"\na.List1.AddItem \"alpha\"\na.List1.AddItem \"beta\"\na.List1.List(0,1)=\"A\"\na.List1.List(1,1)=\"B\"\na.List1.ListIndex=1\nb.List1.AddItem \"separate instance\"\nb.List1.List(0,1)=\"C\"\nb.List1.ListIndex=0\na.Show 0\nb.Show 0\nEnd Sub"
        } else {
            "Sub Main()\nDim a As NativeForm,b As NativeForm\nSet a=New NativeForm\nSet b=New NativeForm\na.Caption=\"First window\"\nb.Caption=\"Second window\"\na.Entry.Text=\"Value one\"\nb.Entry.Text=\"Value two\"\na.Show 0\nb.Show 0\nEnd Sub"
        };
        let main = if combos {
            main.replace("List1", "Combo1")
                .replace("b.Show 0\nEnd Sub", "b.Show 0\na.Combo1.DropDown\nEnd Sub")
        } else {
            main.into()
        };
        ide.add_module("Main.bas", &main)?;
        ide.dispatch(Action::Run, &mut NoHost)?;
        for _ in 0..100 {
            ide.tick(&mut NoHost)?;
            if ide.mode() == Mode::Completed {
                break;
            }
        }
        if ide.form_instances()?.len()
            != if metadata || frames || tabs || pages || images {
                1
            } else {
                2
            }
        {
            return Err("expected two live native form instances".into());
        }
        let mut h = Harness::builder()
            .allow_missing_glyphs()
            .with_size(egui::vec2(width, 1000.))
            .with_pixels_per_point(scale)
            .wgpu()
            .build_ui_state(
                |ui, s: &mut State| {
                    if s.metadata {
                        ui.style_mut().interaction.tooltip_delay = 0.;
                    }
                    let _ = s.view.show(ui, &s.ide, s.theme);
                },
                State {
                    ide,
                    view: EditorUi::default(),
                    theme,
                    metadata,
                },
            );
        h.run_steps(5);
        if metadata {
            h.get(
                egui_kittest::kittest::By::new()
                    .label("Entry")
                    .role(egui::accesskit::Role::TextInput),
            )
            .hover();
            h.run_steps(8);
            if h.query_by_label("A native form control tip").is_none() {
                return Err("expected native tooltip".into());
            }
        }
        let prefix = if images {
            "m7a4-native-images"
        } else if pages {
            "m7a4-native-pages"
        } else if tabs {
            "m7a4-native-tabs"
        } else if frames {
            "m7a4-native-frames"
        } else if metadata {
            "m7a4-native-metadata"
        } else if combos {
            "m7a4-combo-instances"
        } else if lists {
            "m7a4-list-instances"
        } else {
            "m7a4-instances"
        };
        h.render()?.save(format!("{folder}/{prefix}-{name}.png"))?;
    }
    Ok(())
}
