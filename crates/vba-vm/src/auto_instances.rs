//! Lazy project-class construction on canonical variable reads. Inspection stays inert.
use super::*;
impl Session {
    /// Returns true only when an ordinary initializer frame was entered. The
    /// caller retains its instruction/operands until initialization completes.
    pub(super) fn ensure_auto_instance(
        &mut self,
        location: Location,
        source: Option<ir::SourceIndex>,
    ) -> Result<bool> {
        let Some(source) = source else {
            return Ok(false);
        };
        if self.store.read(location)? != &Value::Nothing {
            return Ok(false);
        }
        let object = self.allocate_instance(source)?;
        self.write_value(location, Value::Object(object))?;
        let depth = self.frames.len();
        if let Err(error) = self.initialize_instance(source, object, true) {
            self.store.write(location, Value::Nothing)?;
            self.retire_instance(object)?;
            return Err(error);
        }
        if self.frames.len() > depth {
            self.top_mut()?.class_return = Some(ClassReturn::AutoInitialize {
                object,
                location,
                field: None,
            });
            Ok(true)
        } else {
            Ok(false)
        }
    }
}
