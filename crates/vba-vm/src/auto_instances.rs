//! Lazy project-class construction on canonical variable reads. Inspection stays inert.
use super::*;
impl Session {
    pub(super) fn register_element_reference(
        &mut self,
        owner: Location,
        location: Location,
        kind: ScalarKind,
    ) -> Result<()> {
        if self.kinds.len() >= 65536 {
            return Err(resource());
        }
        self.kinds.insert(location, kind);
        if !self.nominal_types.contains_key(&location)
            && let Some(ty) = self.nominal_types.get(&owner)
        {
            self.charge(nominal_bytes(ty))?;
            if let Some(ty) = self.nominal_types.get(&owner).cloned() {
                self.nominal_types.insert(location, ty);
            }
        }
        Ok(())
    }

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
