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
    let accelerators = std::env::args()
        .nth(2)
        .is_some_and(|o| o == "--accelerators");
    let multiple = std::env::args()
        .nth(2)
        .is_some_and(|o| o == "--multi-select");
    let runtime_controls = std::env::args()
        .nth(2)
        .is_some_and(|o| o == "--runtime-controls");
    let dynamic = std::env::args()
        .nth(2)
        .is_some_and(|o| o.starts_with("--dynamic-"));
    let refedit = std::env::args()
        .nth(2)
        .is_some_and(|o| o == "--refedit-controls");
    let images = std::env::args()
        .nth(2)
        .is_some_and(|o| o == "--image-controls");
    let pages = std::env::args()
        .nth(2)
        .is_some_and(|o| o == "--page-controls" || o == "--dynamic-page-controls");
    let tabs = std::env::args()
        .nth(2)
        .is_some_and(|o| o == "--tab-controls" || o == "--dynamic-tab-controls");
    let combos = std::env::args()
        .nth(2)
        .is_some_and(|option| option == "--combo-controls");
    let lists = std::env::args()
        .nth(2)
        .is_some_and(|option| option == "--list-controls" || option == "--multi-select");
    let frames = std::env::args()
        .nth(2)
        .is_some_and(|o| o == "--frame-controls");
    let metadata = std::env::args().nth(2).is_some_and(|o| o == "--metadata");
    let colors = std::env::args().nth(2).is_some_and(|o| o == "--colors");
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
                bytes: if refedit {
                    fixture::refedit_companion()
                } else if images {
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
                } else if runtime_controls || colors {
                    fixture::companion()
                } else {
                    fixture::boolean_companion()
                },
            }],
        )?;
        let main = if colors {
            "Sub Main()\nDim a As New NativeForm,b As New NativeForm\na.Caption=\"Native RGB colours\"\nb.Caption=\"Separate colour state\"\na.BackColor=RGB(230,240,250)\na.Heading.BackColor=RGB(200,230,190)\na.Heading.ForeColor=RGB(70,20,90)\na.Entry.BackColor=RGB(250,230,200)\na.Entry.ForeColor=RGB(80,30,20)\na.Entry.EnterFieldBehavior=1\na.Apply.BackColor=RGB(190,220,240)\na.Apply.ForeColor=RGB(10,50,100)\na.Show 0\nb.Show 0\nEnd Sub"
        } else if runtime_controls {
            "Sub Main()\nDim group As MSForms.Frame,entry As MSForms.TextBox,list As MSForms.ListBox,button As MSForms.CommandButton\nNativeForm.Caption=\"Runtime native controls\"\nNativeForm.Width=360\nNativeForm.Height=340\nSet group=NativeForm.Controls.Add(\"Forms.Frame.1\",\"RuntimeGroup\")\ngroup.Caption=\"Added through Controls.Add\"\ngroup.Top=100\ngroup.Width=320\ngroup.Height=200\nSet entry=group.Controls.Add(\"Forms.TextBox.1\",\"RuntimeEntry\")\nentry.Text=\"Canonical runtime text\"\nentry.Left=10\nentry.Top=24\nentry.Width=270\nSet list=group.Controls.Add(\"Forms.ListBox.1\",\"RuntimeList\")\nlist.Left=10\nlist.Top=60\nlist.Width=180\nlist.Height=110\nlist.AddItem \"Alpha\"\nlist.AddItem \"Beta\"\nlist.AddItem \"Gamma\"\nlist.MultiSelect=2\nlist.Selected(0)=True\nlist.Selected(2)=True\nSet button=group.Controls.Add(\"Forms.CommandButton.1\",\"RuntimeButton\")\nbutton.Caption=\"Action\"\nbutton.Left=205\nbutton.Top=90\nNativeForm.Show 0\nlist.SetFocus\nlist.ListIndex=1\nEnd Sub"
        } else if multiple {
            "Sub Main()\nDim a As New NativeForm,b As New NativeForm\na.Caption=\"Multi selection\"\nb.Caption=\"Extended selection\"\na.List1.MultiSelect=1\nb.List1.MultiSelect=2\na.List1.Height=100\nb.List1.Height=100\na.List1.AddItem \"Alpha\"\na.List1.AddItem \"Beta\"\na.List1.AddItem \"Gamma\"\nb.List1.AddItem \"Alpha\"\nb.List1.AddItem \"Beta\"\nb.List1.AddItem \"Gamma\"\na.List1.Selected(0)=True\na.List1.Selected(2)=True\nb.List1.Selected(0)=True\nb.List1.Selected(2)=True\na.Show 0\nb.Show 0\nb.List1.SetFocus\nb.List1.ListIndex=1\nEnd Sub"
        } else if refedit {
            "Sub Main()\nNativeForm.Show\nEnd Sub"
        } else if images {
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
        let main = if dynamic {
            let (control, collection) = if pages {
                ("Multi1", "Pages")
            } else {
                ("Tabs1", "Tabs")
            };
            format!(
                "Sub Main()\nDim s As Object\nSet s=NativeForm.{control}\ns.{collection}(0).Index=1\ns.{collection}.Add Name:=\"Runtime\",Caption:=\"Added at runtime\",Index:=0\ns.{collection}.Remove \"BlockedTab\"\ns.Value=2\nNativeForm.Show 0\ns.SetFocus\nEnd Sub"
            )
        } else if combos {
            main.replace("List1", "Combo1")
                .replace("b.Show 0\nEnd Sub", "b.Show 0\na.Combo1.DropDown\nEnd Sub")
        } else {
            main.into()
        };
        let main = if accelerators {
            main.replace("a.Show 0", "a.Heading.Accelerator=\"N\"\na.Apply.Accelerator=\"A\"\na.Check1.Accelerator=\"C\"\na.Toggle1.Accelerator=\"T\"\na.Show 0").replace("b.Show 0", "b.Heading.Accelerator=\"N\"\nb.Apply.Accelerator=\"A\"\nb.Check1.Accelerator=\"C\"\nb.Toggle1.Accelerator=\"T\"\nb.Show 0")
        } else {
            main
        };
        ide.add_module("Main.bas", &main)?;
        ide.dispatch(Action::Run, &mut NoHost)?;
        for _ in 0..1000 {
            ide.tick(&mut NoHost)?;
            if ide.mode() == Mode::Completed {
                break;
            }
        }
        if ide.form_instances()?.len()
            != if metadata || frames || tabs || pages || images || refedit || runtime_controls {
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
                    view: {
                        let mut view = EditorUi::default();
                        view.range_selection_enabled = refedit;
                        view
                    },
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
        let prefix = if colors {
            "m7a4-native-colors"
        } else if runtime_controls {
            "m7a4-runtime-controls"
        } else if multiple {
            "m7a4-multiple-selection"
        } else if dynamic && pages {
            "m7a4-dynamic-pages"
        } else if dynamic {
            "m7a4-dynamic-tabs"
        } else if refedit {
            "m7a4-native-refedit"
        } else if images {
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
        if refedit {
            let s = h.state_mut();
            let owner = *s.ide.form_instances()?.first().ok_or("form instance")?;
            s.ide.form_instance_drop(owner, 12, true, &mut NoHost)?;
            for _ in 0..100 {
                if s.ide.mode() != Mode::Running {
                    break;
                }
                s.ide.tick(&mut NoHost)?;
            }
            h.run_steps(5);
            h.render()?
                .save(format!("{folder}/{prefix}-collapsed-{name}.png"))?;
        }
    }
    Ok(())
}
