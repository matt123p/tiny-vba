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
    let fonts = std::env::args().nth(2).is_some_and(|o| o == "--fonts");
    let appearance = std::env::args().nth(2).is_some_and(|o| o == "--appearance");
    let text_align = std::env::args().nth(2).is_some_and(|o| o == "--text-align");
    let word_wrap = std::env::args().nth(2).is_some_and(|o| o == "--word-wrap");
    let boolean_wrap = std::env::args()
        .nth(2)
        .is_some_and(|o| o == "--boolean-word-wrap");
    let caption_alignment = std::env::args()
        .nth(2)
        .is_some_and(|o| o == "--caption-alignment");
    let auto_tab = std::env::args().nth(2).is_some_and(|o| o == "--auto-tab");
    let hide_selection = std::env::args()
        .nth(2)
        .is_some_and(|o| o == "--hide-selection");
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
                bytes: if hide_selection {
                    fixture::combo_companion_flags(3, 1, 0x0c80_481b)
                } else if caption_alignment {
                    fixture::boolean_alignment_companion(true)
                } else if boolean_wrap {
                    fixture::boolean_word_wrap_companion(0x2c00_081b)
                } else if word_wrap || auto_tab {
                    fixture::companion()
                } else if text_align {
                    fixture::alignment_companion(2)
                } else if appearance {
                    fixture::appearance_companion()
                } else if refedit {
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
                } else if runtime_controls || colors || fonts {
                    fixture::companion()
                } else {
                    fixture::boolean_companion()
                },
            }],
        )?;
        let main = if auto_tab {
            "Sub Main()\nDim a As New NativeForm,b As New NativeForm,c As Object\na.Caption=\"AutoTab enabled\"\nb.Caption=\"AutoTab disabled\"\nSetup a,True\nSetup b,False\na.Show 0\nb.Show 0\nb.Entry.SetFocus\nEnd Sub\nPrivate Sub Setup(ByVal f As MSForms.UserForm,ByVal enabled As Boolean)\nf.Controls(\"Heading\").Caption=\"Enter a three-character code\"\nf.Controls(\"Heading\").Width=220\nf.Controls(\"Entry\").Text=\"ab\"\nf.Controls(\"Entry\").MaxLength=3\nf.Controls(\"Entry\").AutoTab=enabled\nf.Controls(\"Entry\").SelStart=2\nf.Controls(\"Entry\").EnterFieldBehavior=1\nf.Controls(\"Entry\").FontSize=12\nf.Controls(\"Entry\").Width=168\nDim c As Object\nSet c=f.Controls.Add(\"Forms.ComboBox.1\",\"Code\")\nc.Left=16\nc.Top=110\nc.Width=168\nc.Height=28\nc.FontSize=12\nc.AddItem \"abc\"\nc.Text=\"abc\"\nc.SelStart=1\nc.SelLength=2\nc.HideSelection=False\nc.MaxLength=3\nc.AutoTab=enabled\nEnd Sub"
        } else if hide_selection {
            "Sub Main()\nDim a As New NativeForm,b As New NativeForm\na.Caption=\"Selection remains visible\"\nb.Caption=\"Selection hides without focus\"\nSetup a,False\nSetup b,True\na.Show 0\nb.Show 0\nb.Apply.SetFocus\nEnd Sub\nPrivate Sub Setup(ByVal f As MSForms.UserForm,ByVal hide As Boolean)\nf.Height=200\nf.Controls(\"Heading\").Visible=False\nf.Controls(\"Apply\").Top=155\nDim c As Object\nSet c=f.Controls(\"Entry\")\nc.Left=16\nc.Top=12\nc.Width=190\nc.Height=36\nc.FontSize=12\nc.EnterFieldBehavior=1\nc.Text=\"ALPHA BRAVO\"\nc.HideSelection=hide\nc.SelStart=1\nc.SelLength=4\nSet c=f.Controls(\"Combo1\")\nc.Left=16\nc.Top=65\nc.Width=190\nc.Height=28\nc.FontSize=12\nc.EnterFieldBehavior=1\nc.Text=\"ALPHA BRAVO\"\nc.HideSelection=hide\nc.SelStart=1\nc.SelLength=4\nSet c=f.Controls.Add(\"Forms.ComboBox.1\",\"Fixed\")\nc.Left=16\nc.Top=106\nc.Width=190\nc.Height=28\nc.FontSize=12\nc.Style=2\nc.AddItem \"ALPHA BRAVO\"\nc.ListIndex=0\nc.EnterFieldBehavior=1\nc.HideSelection=hide\nc.SelStart=1\nc.SelLength=4\nEnd Sub"
        } else if caption_alignment {
            "Sub Main()\nDim a As New NativeForm,b As New NativeForm\na.Caption=\"Caption left of indicator\"\nb.Caption=\"Caption right of indicator\"\nSetup a,fmAlignmentLeft\nSetup b,fmAlignmentRight\na.Show 0\nb.Show 0\nEnd Sub\nPrivate Sub Setup(ByVal f As MSForms.UserForm,ByVal side As Long)\nf.Height=230\nf.Controls(\"Heading\").Visible=False\nf.Controls(\"Entry\").Visible=False\nf.Controls(\"Apply\").Visible=False\nf.Controls(\"Toggle1\").Visible=False\nDim name As Variant,c As Object,n As Long\nFor Each name In Array(\"Check1\",\"ChoiceA\",\"ChoiceB\")\nSet c=f.Controls(name)\nc.Caption=\"Caption stays inside its own bounds\"\nc.Left=16\nc.Top=12+n*68\nc.Width=145\nc.Height=60\nc.FontSize=12\nc.Alignment=side\nc.WordWrap=n<>1\nn=n+1\nNext name\nEnd Sub"
        } else if boolean_wrap {
            "Sub Main()\nDim a As New NativeForm,b As New NativeForm\na.Caption=\"Boolean WordWrap enabled\"\nb.Caption=\"Boolean WordWrap disabled\"\nSetup a,True\nSetup b,False\na.Show 0\nb.Show 0\nEnd Sub\nPrivate Sub Setup(ByVal f As MSForms.UserForm,ByVal wrap As Boolean)\nf.Height=230\nf.Controls(\"Heading\").Visible=False\nf.Controls(\"Entry\").Visible=False\nf.Controls(\"Apply\").Visible=False\nf.Controls(\"ChoiceB\").Visible=False\nDim name As Variant,c As Object,n As Long\nFor Each name In Array(\"Check1\",\"ChoiceA\",\"Toggle1\")\nSet c=f.Controls(name)\nc.Caption=\"Caption wraps inside fixed bounds\"\nc.Left=16\nc.Top=12+n*68\nc.Width=110\nc.Height=60\nc.FontSize=12\nc.WordWrap=wrap\nn=n+1\nNext name\nEnd Sub"
        } else if word_wrap {
            "Sub Main()\nDim a As New NativeForm,b As New NativeForm,c As Object\na.Caption=\"Native WordWrap enabled\"\nb.Caption=\"Native WordWrap disabled\"\nFor Each c In a.Controls\nc.FontSize=12\nNext c\nFor Each c In b.Controls\nc.FontSize=12\nNext c\na.Heading.Caption=\"Caption wraps inside fixed bounds\"\nb.Heading.Caption=a.Heading.Caption\na.Heading.Top=12\nb.Heading.Top=12\na.Heading.Width=110\nb.Heading.Width=110\na.Heading.Height=40\nb.Heading.Height=40\na.Heading.WordWrap=True\nb.Heading.WordWrap=False\na.Entry.Text=\"Text wraps inside fixed bounds\"\nb.Entry.Text=a.Entry.Text\na.Entry.Top=65\nb.Entry.Top=65\na.Entry.Width=168\nb.Entry.Width=168\na.Entry.Height=55\nb.Entry.Height=55\na.Entry.MultiLine=True\nb.Entry.MultiLine=True\na.Entry.WordWrap=True\nb.Entry.WordWrap=False\na.Apply.Caption=\"Wrapped button caption\"\nb.Apply.Caption=a.Apply.Caption\na.Apply.Top=140\nb.Apply.Top=140\na.Apply.Width=110\nb.Apply.Width=110\na.Apply.Height=40\nb.Apply.Height=40\na.Apply.WordWrap=True\nb.Apply.WordWrap=False\na.Show 0\nb.Show 0\nEnd Sub"
        } else if text_align {
            "Sub Main()\nDim a As New NativeForm,b As New NativeForm,rows As MSForms.ListBox,choices As MSForms.ComboBox\na.Caption=\"Left and centre\"\nb.Caption=\"Right and popup rows\"\na.Heading.Caption=\"Left aligned caption\"\na.Heading.Width=168\na.Heading.TextAlign=1\na.Entry.Text=\"Centred text\"\na.Entry.Width=168\na.Entry.FontSize=12\na.Entry.TextAlign=2\nb.Heading.Caption=\"Right aligned caption\"\nb.Heading.Width=168\nb.Heading.TextAlign=3\nb.Entry.Text=\"Right aligned text\"\nb.Entry.Width=168\nb.Entry.FontSize=12\nb.Entry.TextAlign=3\nSet rows=a.Controls.Add(\"Forms.ListBox.1\",\"Rows\")\nrows.Left=16\nrows.Top=110\nrows.Width=250\nrows.Height=60\nrows.ColumnCount=2\nrows.FontSize=12\nrows.AddItem \"First column\"\nrows.List(0,1)=\"Second column\"\nrows.TextAlign=2\nSet choices=b.Controls.Add(\"Forms.ComboBox.1\",\"Choices\")\nchoices.Left=16\nchoices.Top=125\nchoices.Width=250\nchoices.Style=2\nchoices.FontSize=12\nchoices.AddItem \"Right edit value\"\nchoices.AddItem \"Left popup row\"\nchoices.ListIndex=0\nchoices.TextAlign=3\na.Show 0\nb.Show 0\nchoices.SetFocus\nchoices.DropDown\nEnd Sub"
        } else if appearance {
            "Sub Main()\nDim a As New NativeForm,b As New NativeForm\na.Caption=\"Native borders and transparency\"\nb.Caption=\"Native system-colour effects\"\na.BackColor=RGB(230,240,250)\na.Heading.Caption=\"Transparent label\"\na.Heading.Width=168\na.Heading.BackStyle=0\na.Heading.ForeColor=RGB(40,50,60)\na.Heading.BorderStyle=0\na.Entry.BackStyle=1\na.Entry.BackColor=RGB(250,230,200)\na.Entry.ForeColor=RGB(40,50,60)\na.Entry.BorderColor=RGB(80,30,20)\na.Entry.BorderStyle=1\na.Entry.FontSize=12.25\na.Entry.FontItalic=True\na.Apply.BackStyle=0\na.Apply.ForeColor=RGB(40,50,60)\nb.BorderStyle=0\nb.Heading.Width=168\nb.Heading.Caption=\"Etched and raised edges\"\nb.Heading.SpecialEffect=1\nb.Heading.BackStyle=1\nb.Entry.BackStyle=1\nb.Entry.SpecialEffect=3\nb.Entry.FontSize=12.25\nDim flat As MSForms.ListBox,edged As MSForms.ListBox\nSet flat=a.Controls.Add(\"Forms.ListBox.1\",\"FlatRows\")\nflat.Left=145\nflat.Top=70\nflat.Width=110\nflat.Height=90\nflat.BackColor=RGB(31,53,79)\nflat.ForeColor=RGB(220,230,240)\nflat.SpecialEffect=0\nflat.BorderStyle=0\nflat.AddItem \"Flat list\"\nflat.AddItem \"No outside outline\"\nSet edged=b.Controls.Add(\"Forms.ListBox.1\",\"EdgedRows\")\nedged.Left=145\nedged.Top=70\nedged.Width=110\nedged.Height=90\nedged.SpecialEffect=2\nedged.AddItem \"Sunken list\"\na.Show 0\nb.Show 0\nEnd Sub"
        } else if fonts {
            "Sub Main()\nDim a As New NativeForm,b As New NativeForm\na.Caption=\"Native font attributes\"\nb.Caption=\"Independent default font\"\na.Entry.Height=48\na.Apply.Top=120\na.Entry.EnterFieldBehavior=1\na.Entry.Font.Size=18.5\na.Entry.Font.Italic=True\na.Entry.Font.Underline=True\na.Entry.Font.Strikethrough=True\na.Apply.Font.Size=14\nSet a.Heading.Font=a.Font\na.Heading.Font.Size=12\na.Heading.Font.Underline=True\na.Show 0\nb.Show 0\nEnd Sub"
        } else if colors {
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
        let prefix = if auto_tab {
            "m7a4-native-auto-tab"
        } else if hide_selection {
            "m7a4-native-hide-selection"
        } else if caption_alignment {
            "m7a4-native-caption-alignment"
        } else if boolean_wrap {
            "m7a4-boolean-word-wrap"
        } else if word_wrap {
            "m7a4-native-word-wrap"
        } else if text_align {
            "m7a4-native-text-align"
        } else if appearance {
            "m7a4-native-appearance"
        } else if fonts {
            "m7a4-native-fonts"
        } else if colors {
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
