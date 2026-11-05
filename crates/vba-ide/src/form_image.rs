//! Native pictures are ordinary VM objects naming immutable source resources.
use crate::*;
pub(crate) const PICTURE: &str = r#"Option Explicit
Public VbaSource As Long
Public VbaControl As Long
Public VbaWidth As Long
Public VbaHeight As Long
Public VbaType As Integer
Public Property Get Width() As Long
Width=VbaWidth
End Property
Public Property Get Height() As Long
Height=VbaHeight
End Property
Public Property Get Type() As Integer
Type=VbaType
End Property
"#;
pub(crate) const IMAGE: &str = r#"
Public VbaPicture As Object
Public VbaPictureSizeMode As Integer
Public VbaPictureAlignment As Integer
Public PictureTiling As Boolean
Public VbaAutoSize As Boolean
Public VbaBorderStyle As Integer
Public VbaSpecialEffect As Integer
Public Property Get Picture() As stdole.IPictureDisp
Attribute Picture.VB_UserMemId = 0
Set Picture=VbaPicture
End Property
Public Property Set Picture(ByVal value As stdole.IPictureDisp)
VbaValidateSize value,VbaAutoSize
Set VbaPicture=value
VbaFitPicture
End Property
Public Property Get AutoSize() As Boolean
AutoSize=VbaAutoSize
End Property
Public Property Let AutoSize(ByVal value As Boolean)
VbaValidateSize VbaPicture,value
VbaAutoSize=value
VbaFitPicture
End Property
Private Sub VbaValidateSize(ByVal picture As stdole.IPictureDisp,ByVal enabled As Boolean)
If Not enabled Then Exit Sub
If picture Is Nothing Then Exit Sub
Dim width As Double,height As Double
width=CDbl(picture.Width)*72/2540
height=CDbl(picture.Height)*72/2540
If width<0 Or height<0 Or width>32767 Or height>32767 Then Err.Raise 380
End Sub
Public Sub VbaFitPicture()
VbaValidateSize VbaPicture,VbaAutoSize
If Not VbaAutoSize Then Exit Sub
If VbaPicture Is Nothing Then Exit Sub
VbaWidth=CSng(VbaPicture.Width)*72/2540
VbaHeight=CSng(VbaPicture.Height)*72/2540
End Sub
Public Property Get PictureSizeMode() As Integer
PictureSizeMode=VbaPictureSizeMode
End Property
Public Property Let PictureSizeMode(ByVal value As Integer)
If value<>0 And value<>1 And value<>3 Then Err.Raise 380
VbaPictureSizeMode=value
End Property
Public Property Get PictureAlignment() As Integer
PictureAlignment=VbaPictureAlignment
End Property
Public Property Let PictureAlignment(ByVal value As Integer)
If value<0 Or value>4 Then Err.Raise 380
VbaPictureAlignment=value
End Property
Public Property Get BorderStyle() As Integer
BorderStyle=VbaBorderStyle
End Property
Public Property Let BorderStyle(ByVal value As Integer)
If value<0 Or value>1 Then Err.Raise 380
VbaBorderStyle=value
End Property
Public Property Get SpecialEffect() As Integer
SpecialEffect=VbaSpecialEffect
End Property
Public Property Let SpecialEffect(ByVal value As Integer)
If value<>0 And value<>1 And value<>2 And value<>3 And value<>6 Then Err.Raise 380
VbaSpecialEffect=value
End Property
Public Event Click
Public Sub VbaClick()
RaiseEvent Click
End Sub
"#;
impl Ide {
    /// Inspect canonical picture ownership without loading forms or invoking getters.
    pub fn form_instance_picture(
        &self,
        instance: FormInstance,
        control: u32,
    ) -> Result<Option<&vba_project::forms::Picture>> {
        let value = self.form_instance_field(instance, Some(control), "Picture")?;
        if value == Value::Nothing {
            return Ok(None);
        }
        let Value::Object(picture) = value else {
            return Err(error("invalid picture object"));
        };
        let session = self
            .session
            .as_ref()
            .ok_or_else(|| error("no form session"))?;
        let Value::Long(source) = session.read_instance_field(picture, "VbaSource")? else {
            return Err(error("picture source"));
        };
        let Value::Long(id) = session.read_instance_field(picture, "VbaControl")? else {
            return Err(error("picture resource"));
        };
        let source = u64::try_from(source).map_err(|_| error("picture source"))?;
        let id = u32::try_from(id).map_err(|_| error("picture resource"))?;
        let resource = self
            .native_forms
            .get(&SourceId(source))
            .and_then(|f| f.design.controls.iter().find(|c| c.id == id))
            .and_then(|c| c.image.as_ref())
            .and_then(|i| i.picture.as_ref())
            .ok_or_else(|| error("missing native picture resource"))?;
        Ok(Some(resource))
    }
}
