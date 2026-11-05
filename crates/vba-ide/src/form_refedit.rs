//! Native Excel RefEdit contracts; worksheet selection itself belongs to the embedding.
pub(crate) fn event(kind: crate::forms::ControlKind, name: &str) -> Option<&'static str> {
    (kind == crate::forms::ControlKind::RefEdit)
        .then(|| {
            ["Change", "DropButtonClick"]
                .into_iter()
                .find(|event| event.eq_ignore_ascii_case(name))
        })
        .flatten()
}
pub(crate) const MEMBERS: &str = "Public VbaDropVisible As Boolean\nPublic Event DropButtonClick\nPublic Sub VbaSetDrop(ByVal opened As Boolean)\nIf VbaDropVisible=opened Then Exit Sub\nVbaDropVisible=opened\nRaiseEvent DropButtonClick\nEnd Sub\n";

impl crate::Ide {
    /// Current native RefEdit requesting worksheet selection; never runs code or loads forms.
    pub fn form_range_selection(&self) -> crate::Result<Option<(crate::FormInstance, u32)>> {
        for instance in self.form_instances()? {
            if !self.form_instance_visible(instance)
                || self
                    .form_gate
                    .waits
                    .iter()
                    .next_back()
                    .map(|(_, owner)| *owner)
                    != Some(instance.object)
            {
                continue;
            }
            let Some(native) = self.native_form(instance.source) else {
                continue;
            };
            for control in &native.design.controls {
                if control.kind == vba_project::forms::Kind::RefEdit
                    && self.form_instance_field(instance, Some(control.id), "VbaDropVisible")?
                        == crate::Value::Boolean(true)
                {
                    return Ok(Some((instance, control.id)));
                }
            }
        }
        Ok(None)
    }
}
