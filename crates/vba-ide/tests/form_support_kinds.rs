//! A project gets support classes only for the control kinds it can create.
//! Code can still declare the other MSForms control types.
use vba_ide::forms::{Bounds, ControlKind, DesignEdit};
use vba_ide::{Ide, Mode, NoHost};
type R<T = ()> = Result<T, Box<dyn std::error::Error>>;

#[test]
fn a_form_without_a_control_kind_still_declares_its_type() -> R {
    let mut ide = Ide::new();
    let id = ide.add_form("Plain")?;
    for (kind, top) in [
        (ControlKind::Label, 12.0),
        (ControlKind::CommandButton, 42.0),
    ] {
        ide.edit_form(
            id,
            DesignEdit::Add(
                kind,
                Bounds {
                    left: 12.0,
                    top,
                    width: 120.0,
                    height: 24.0,
                },
            ),
        )?;
    }
    // `Controls` without `Add`: the project cannot create controls.
    ide.set_text(
        "Attribute VB_Name = \"Plain\"\r\n\
         Private Sub UserForm_Initialize()\n\
         Dim c As Object,box As MSForms.CheckBox,caption As MSForms.Label\n\
         For Each c In Me.Controls\n\
         If TypeName(c)=\"Label\" Then Set caption=c\n\
         Debug.Print TypeName(c)\n\
         Next c\n\
         Debug.Print caption.Name & \" \" & (box Is Nothing)\n\
         End Sub\n"
            .into(),
    )?;
    ide.run_form(id, false, &mut NoHost)?;
    for _ in 0..2000 {
        if ide.mode() != Mode::Running {
            break;
        }
        ide.tick(&mut NoHost)?;
    }
    assert_eq!(ide.mode(), Mode::Completed, "{:?}", ide.diagnostics);
    assert_eq!(ide.output(), ["Label", "CommandButton", "Label1 True"]);
    Ok(())
}
