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
    let images = option.as_deref() == Some("--image-controls");
    let pages = option.as_deref() == Some("--page-controls");
    let tabs = option.as_deref() == Some("--tab-controls");
    let frames = option.as_deref() == Some("--frame-controls");
    let metadata = option.as_deref() == Some("--metadata");
    let combo = option.as_deref() == Some("--combo-controls");
    let list = option.as_deref() == Some("--list-controls");
    let numeric = option.as_deref() == Some("--numeric-controls");
    let focus = option.as_deref() == Some("--focus-events");
    let text_controls = option.as_deref() == Some("--text-controls");
    let password = option.as_deref() == Some("--password-text");
    let boolean = match option.as_deref() {
        None => false,
        Some("--boolean-controls") => true,
        Some(
            "--focus-events" | "--text-controls" | "--password-text" | "--numeric-controls"
            | "--list-controls" | "--combo-controls" | "--metadata" | "--frame-controls"
            | "--tab-controls" | "--page-controls" | "--image-controls",
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
    } else if list {
        "Stop\r\nDebug.Print List1.ListIndex\r\nDebug.Print List1.Value"
    } else if numeric {
        "Stop\r\nDebug.Print Spin1.Value\r\nDebug.Print Scroll1.Value"
    } else if boolean {
        "Stop\r\nDebug.Print Check1.Value\r\nDebug.Print ChoiceA.Value\r\nDebug.Print ChoiceB.Value\r\nDebug.Print Toggle1.Value"
    } else if text_controls {
        "Stop\r\nDebug.Print Len(Entry.Text)\r\nDebug.Print Entry.SelStart\r\nDebug.Print Entry.SelLength"
    } else if focus {
        "Debug.Print Entry.Text"
    } else {
        "Stop\r\nDebug.Print Entry.Text"
    };
    let mut text = String::from_utf8(fixture::export())?.replace("Debug.Print Entry.Text", body);
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
        if images {
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
