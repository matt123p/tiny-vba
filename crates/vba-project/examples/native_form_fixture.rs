//! Maintenance fixture generator; emits independently authored input, never Office output.
#[path = "../tests/support/native_forms.rs"]
mod fixture;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let dir = std::path::PathBuf::from(
        std::env::args()
            .nth(1)
            .ok_or("supply a fixture output directory")?,
    );
    std::fs::create_dir_all(&dir)?;
    let option = std::env::args().nth(2);
    let refedit = option.as_deref() == Some("--refedit-controls");
    let images = option.as_deref() == Some("--image-controls");
    let pages = option.as_deref() == Some("--page-controls");
    let tabs = option.as_deref() == Some("--tab-controls");
    let frames = option.as_deref() == Some("--frame-controls");
    let metadata = option.as_deref() == Some("--metadata");
    let window_activation = option.as_deref() == Some("--window-activation");
    let fonts = option.as_deref() == Some("--fonts");
    let colors = option.as_deref() == Some("--colors");
    let nested_mouse = option.as_deref() == Some("--nested-mouse");
    let composite_mouse = option.as_deref() == Some("--composite-mouse");
    let combo = option.as_deref() == Some("--combo-controls") || composite_mouse;
    let multiple = option.as_deref() == Some("--multi-select");
    let list = option.as_deref() == Some("--list-controls") || multiple;
    let numeric = option.as_deref() == Some("--numeric-controls");
    let focus = option.as_deref() == Some("--focus-events");
    let keyboard = option.as_deref() == Some("--keyboard-events");
    let mouse = option.as_deref() == Some("--mouse-events");
    let text_mouse = option.as_deref() == Some("--text-mouse");
    let accelerators = option.as_deref() == Some("--accelerators");
    let routing = option.as_deref() == Some("--keyboard-routing");
    let text_controls = option.as_deref() == Some("--text-controls");
    let password = option.as_deref() == Some("--password-text");
    let boolean = match option.as_deref() {
        None => false,
        Some("--boolean-controls") => true,
        Some(
            "--focus-events"
            | "--text-controls"
            | "--password-text"
            | "--numeric-controls"
            | "--list-controls"
            | "--multi-select"
            | "--combo-controls"
            | "--metadata"
            | "--frame-controls"
            | "--tab-controls"
            | "--page-controls"
            | "--image-controls"
            | "--refedit-controls"
            | "--keyboard-events"
            | "--keyboard-routing"
            | "--accelerators"
            | "--mouse-events"
            | "--text-mouse"
            | "--composite-mouse"
            | "--nested-mouse"
            | "--window-activation"
            | "--colors"
            | "--fonts",
        ) => false,
        Some(_) => return Err("unknown fixture option".into()),
    };
    let body = if images {
        "Stop\r\nDebug.Print Image1.PictureAlignment\r\nDebug.Print Image2.Picture Is Image1.Picture"
    } else if pages {
        "Stop\r\nDebug.Print Multi1.Value\r\nDebug.Print Multi1.SelectedItem.Caption"
    } else if tabs {
        "Stop\r\nDebug.Print Tabs1.Value\r\nDebug.Print Tabs1.SelectedItem.Caption"
    } else if combo {
        "Stop\r\nDebug.Print Combo1.ListIndex\r\nDebug.Print Combo1.Value"
    } else if multiple {
        "Stop\r\nDebug.Print List1.ListIndex\r\nDebug.Print List1.Selected(0)\r\nDebug.Print List1.Selected(1)\r\nDebug.Print List1.Selected(2)\r\nDebug.Print IsNull(List1.Value)\r\nDebug.Print TypeName(List1)"
    } else if list {
        "Stop\r\nDebug.Print List1.ListIndex\r\nDebug.Print List1.Value"
    } else if numeric {
        "Stop\r\nDebug.Print Spin1.Value\r\nDebug.Print Scroll1.Value"
    } else if boolean {
        "Stop\r\nDebug.Print Check1.Value\r\nDebug.Print ChoiceA.Value\r\nDebug.Print ChoiceB.Value\r\nDebug.Print Toggle1.Value"
    } else if text_controls {
        "Stop\r\nDebug.Print Len(Entry.Text)\r\nDebug.Print Entry.SelStart\r\nDebug.Print Entry.SelLength"
    } else if nested_mouse {
        "Debug.Print Me.Caption"
    } else if text_mouse {
        "Debug.Print Entry.Text\r\nDebug.Print Entry.SelLength"
    } else if focus || mouse {
        "Debug.Print Entry.Text"
    } else {
        "Stop\r\nDebug.Print Entry.Text"
    };
    let mut text = String::from_utf8(fixture::export())?.replace("Debug.Print Entry.Text", body);
    if fonts {
        let code="Attribute VB_Name = \"NativeForm\"\nAttribute VB_PredeclaredId = True\nOption Explicit\nPrivate Sub UserForm_Initialize()\nHeading.Font.Size=12\nSet Entry.Font=Heading.Font\nEntry.Font.Size=12.25\nEntry.Font.Italic=True\nEntry.Font.Underline=True\nEntry.EnterFieldBehavior=1\nApply.Font.Size=11\nDebug.Print \"fonts ready\"\nEnd Sub\nPrivate Sub Apply_Click()\nEntry.Font.Weight=700\nDebug.Print Entry.Font.Weight\nStop\nEntry.Font.Size=14.5\nDebug.Print Entry.Font.Size\nDebug.Print Entry.Font Is Heading.Font\nMe.Hide\nEnd Sub\n".replace('\n',"\r\n");
        text = String::from_utf8(fixture::export())?.replace(fixture::SOURCE, &code);
    }
    if colors {
        let code = "Attribute VB_Name = \"NativeForm\"\nAttribute VB_PredeclaredId = True\nOption Explicit\nPrivate Sub UserForm_Initialize()\nMe.BackColor=RGB(230,240,250)\nHeading.BackColor=RGB(200,230,190)\nEntry.EnterFieldBehavior=1\nEntry.ForeColor=RGB(80,30,20)\nApply.BackColor=RGB(190,220,240)\nDebug.Print \"colours ready\"\nEnd Sub\nPrivate Sub Apply_Click()\nEntry.BackColor=RGB(250,230,200)\nDebug.Print Entry.BackColor\nStop\nEntry.ForeColor=RGB(1,2,3)\nDebug.Print Entry.ForeColor\nMe.Hide\nEnd Sub\n".replace('\n',"\r\n");
        text = String::from_utf8(fixture::export())?.replace(fixture::SOURCE, &code);
    }
    if window_activation {
        text = String::from_utf8(fixture::export())?.replace(
            fixture::SOURCE,
            &r#"Attribute VB_Name = "NativeForm"
Attribute VB_PredeclaredId = True
Option Explicit
Private started As Boolean
Private Sub UserForm_Activate()
Debug.Print Me.Caption & " active"
If Not started And VBA.UserForms.Count=1 Then
started=True
Me.Hide
Me.Caption="A"
Me.Show 0
Dim child As NativeForm
Set child=New NativeForm
child.Caption="B"
child.Show 0
End If
End Sub
Private Sub UserForm_Deactivate()
Debug.Print Me.Caption & " inactive"
End Sub
Private Sub Apply_Click()
Debug.Print Me.Caption & " clicked"
Me.Hide
End Sub
"#
            .replace("\n", "\r\n"),
        );
    }
    if nested_mouse {
        text = text.replace(
            "Debug.Print \"initialized\"",
            "Me.Caption=\"Parent\"\r\nDebug.Print \"initialized\"",
        );
        text.push_str("\r\nPrivate Sub Apply_MouseDown(ByVal Button As Integer,ByVal Shift As Integer,ByVal X As Single,ByVal Y As Single)\r\nDim child As NativeForm\r\nIf Me.Caption=\"Parent\" Then\r\nSet child=New NativeForm\r\nchild.Caption=\"Child\"\r\nchild.Show\r\nElse\r\nDebug.Print \"child down\"\r\nStop\r\nEnd If\r\nEnd Sub\r\n");
    }
    if composite_mouse {
        text.push_str("\r\nPrivate Sub Combo1_MouseUp(ByVal Button As Integer,ByVal Shift As Integer,ByVal X As Single,ByVal Y As Single)\r\nDebug.Print \"up\"\r\nStop\r\nEnd Sub\r\nPrivate Sub Combo1_DropButtonClick()\r\nDebug.Print \"drop\"\r\nEnd Sub\r\n");
    }
    if text_mouse {
        text.push_str("\r\nPrivate Sub Entry_MouseDown(ByVal Button As Integer,ByVal Shift As Integer,ByVal X As Single,ByVal Y As Single)\r\nDebug.Print \"down\"\r\nStop\r\nEnd Sub\r\nPrivate Sub Entry_MouseUp(ByVal Button As Integer,ByVal Shift As Integer,ByVal X As Single,ByVal Y As Single)\r\nDebug.Print Entry.SelLength\r\nEnd Sub\r\n");
    }
    if mouse {
        text.push_str("\r\nPrivate Sub Apply_MouseDown(ByVal Button As Integer,ByVal Shift As Integer,ByVal X As Single,ByVal Y As Single)\r\nDebug.Print \"down\"\r\nStop\r\nEnd Sub\r\nPrivate Sub Apply_MouseUp(ByVal Button As Integer,ByVal Shift As Integer,ByVal X As Single,ByVal Y As Single)\r\nDebug.Print \"up\"\r\nEnd Sub\r\n");
    }
    if accelerators {
        text = text.replace(
            "Debug.Print \"initialized\"",
            "Heading.Accelerator=\"N\"\r\nApply.Accelerator=\"A\"\r\nDebug.Print \"initialized\"",
        );
    }
    if routing {
        text.push_str("\r\nPrivate Sub Entry_KeyDown(ByVal KeyCode As MSForms.ReturnInteger,ByVal Shift As Integer)\r\nIf KeyCode.Value=9 Then\r\nDebug.Print \"down\"\r\nStop\r\nEnd If\r\nEnd Sub\r\nPrivate Sub Apply_KeyUp(ByVal KeyCode As MSForms.ReturnInteger,ByVal Shift As Integer)\r\nIf KeyCode.Value=9 Then\r\nDebug.Print \"up\"\r\nElse\r\nDebug.Print \"button up\"\r\nEnd If\r\nEnd Sub\r\nPrivate Sub Apply_KeyDown(ByVal KeyCode As MSForms.ReturnInteger,ByVal Shift As Integer)\r\nDebug.Print \"button down\"\r\nEnd Sub\r\nPrivate Sub Apply_KeyPress(ByVal KeyANSI As MSForms.ReturnInteger)\r\nDebug.Print \"button press\"\r\nKeyANSI.Value=0\r\nEnd Sub\r\n");
    }
    if keyboard {
        text = text.replace(
            "Debug.Print \"initialized\"",
            "Entry.Text=\"\"\r\nDebug.Print \"initialized\"",
        );
        text.push_str("\r\nPrivate Sub Entry_KeyPress(ByVal KeyANSI As MSForms.ReturnInteger)\r\nDebug.Print KeyANSI.Value\r\nStop\r\nKeyANSI.Value=90\r\nEnd Sub\r\n");
    }
    if metadata {
        text = text.replace(
            " ClientHeight",
            " Tag = \"form metadata\"\r\n HelpContextID = 7\r\n ClientHeight",
        );
    }
    if images {
        text.push_str("\r\nPrivate Sub Image1_Click()\r\nSet Image2.Picture=Image1.Picture\r\nImage1.PictureAlignment=fmPictureAlignmentTopLeft\r\nEnd Sub\r\n");
    }
    if pages {
        text.push_str("\r\nPrivate Sub UserForm_Activate()\r\nMulti1.SetFocus\r\nEnd Sub\r\n");
    }
    if tabs {
        text.push_str("\r\nPrivate Sub UserForm_Activate()\r\nTabs1.SetFocus\r\nEnd Sub\r\n");
    }
    if focus {
        text = text.replace(
            "Option Explicit",
            "Option Explicit\r\nPrivate attempts As Long",
        );
        text.push_str("\r\nPrivate Sub Entry_BeforeUpdate(ByVal Cancel As MSForms.ReturnBoolean)\r\nattempts=attempts+1\r\nDebug.Print \"before\"\r\nStop\r\nIf attempts=1 Then Cancel=True\r\nEnd Sub\r\nPrivate Sub Entry_AfterUpdate()\r\nDebug.Print \"after\"\r\nEnd Sub\r\nPrivate Sub Entry_Exit(ByVal Cancel As MSForms.ReturnBoolean)\r\nDebug.Print \"exit\"\r\nEnd Sub\r\nPrivate Sub Apply_Enter()\r\nDebug.Print \"enter\"\r\nEnd Sub\r\n");
    }
    if text_controls {
        text=text.replace("Debug.Print \"initialized\"", "Entry.Text=\"\"\r\nEntry.Height=45\r\nEntry.Width=90\r\nApply.Top=110\r\nDebug.Print \"initialized\"");
    }
    if list {
        text=text.replace("Debug.Print \"initialized\"","List1.AddItem \"alpha\"\r\nList1.AddItem \"beta\"\r\nList1.AddItem \"gamma\"\r\nList1.List(0,1)=\"A\"\r\nList1.List(1,1)=\"B\"\r\nList1.List(2,1)=\"C\"\r\nList1.Left=120\r\nList1.Top=110\r\nList1.Width=160\r\nList1.Height=90\r\nDebug.Print \"initialized\"");
    }
    if combo {
        text=text.replace("Debug.Print \"initialized\"","Combo1.AddItem \"Alpha\"\r\nCombo1.AddItem \"Beta\"\r\nCombo1.List(0,1)=\"A\"\r\nCombo1.List(1,1)=\"B\"\r\nCombo1.Left=120\r\nCombo1.Top=110\r\nCombo1.Width=160\r\nDebug.Print \"initialized\"");
    }
    if numeric {
        text=text.replace("Debug.Print \"initialized\"","Spin1.Left=120\r\nSpin1.Height=24\r\nScroll1.Left=120\r\nScroll1.Top=150\r\nScroll1.Width=120\r\nDebug.Print \"initialized\"");
    }
    std::fs::write(dir.join("NativeForm.frm"), text)?;
    std::fs::write(
        dir.join("NativeForm.frx"),
        if fonts {
            fixture::font_companion()
        } else if refedit {
            fixture::refedit_companion()
        } else if images {
            fixture::image_companion()
        } else if pages {
            fixture::multipage_companion(0)
        } else if tabs {
            fixture::tab_companion(0)
        } else if frames {
            fixture::frame_companion(2)
        } else if metadata {
            fixture::metadata_companion()
        } else if combo {
            fixture::combo_companion(3, 1)
        } else if multiple {
            fixture::list_selection_companion(2)
        } else if list {
            fixture::list_companion()
        } else if numeric {
            fixture::numeric_companion()
        } else if boolean {
            fixture::boolean_companion()
        } else if text_controls {
            fixture::text_companion(0xacf0481b, 0)
        } else if password {
            fixture::text_companion(0x2c80081b, 42)
        } else {
            fixture::companion()
        },
    )?;
    Ok(())
}
