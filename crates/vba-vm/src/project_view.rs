//! Host-neutral transient views over canonical class fields; no host object model.
use super::{Instance, ir};
use std::cell::Cell;
use vba_foreign::{ForeignError, ProjectObjectView};
use vba_values::{Locations, ObjectRef, Value};
pub(super) struct View<'a> {
    pub instances: &'a [Instance],
    pub classes: &'a [ir::Class],
    pub store: &'a Locations,
    pub work: &'a Cell<u64>,
    pub payload: &'a Cell<usize>,
    pub work_limit: u64,
    pub payload_limit: usize,
}
fn refusal(number: i32, description: &str) -> ForeignError {
    ForeignError {
        number,
        description: description.into(),
    }
}
impl View<'_> {
    fn charge(&self, work: usize, payload: usize) -> Result<(), ForeignError> {
        let work = self
            .work
            .get()
            .checked_add(
                u64::try_from(work).map_err(|_| refusal(7, "project inspection work limit"))?,
            )
            .filter(|n| *n <= self.work_limit)
            .ok_or_else(|| refusal(7, "project inspection work limit"))?;
        let payload = self
            .payload
            .get()
            .checked_add(payload)
            .filter(|n| *n <= self.payload_limit)
            .ok_or_else(|| refusal(7, "project inspection payload limit"))?;
        self.work.set(work);
        self.payload.set(payload);
        Ok(())
    }
}
impl ProjectObjectView for View<'_> {
    fn instances(&self, class: &str) -> Result<Vec<ObjectRef>, ForeignError> {
        if class.len() > 128 {
            return Err(refusal(5, "project class name limit"));
        }
        self.charge(self.classes.len() + self.instances.len(), 0)?;
        let class = self
            .classes
            .iter()
            .find(|c| c.name.eq_ignore_ascii_case(class))
            .ok_or_else(|| refusal(438, "unknown project class"))?;
        let objects: Vec<_> = self
            .instances
            .iter()
            .filter(|i| i.alive && !i.terminating && i.source == class.source)
            .map(|i| i.object)
            .collect();
        self.charge(0, objects.len() * std::mem::size_of::<ObjectRef>())?;
        Ok(objects)
    }
    fn public_field(&self, object: ObjectRef, field: &str) -> Result<Value, ForeignError> {
        if field.len() > 128 {
            return Err(refusal(5, "project field name limit"));
        }
        self.charge(self.instances.len() + self.classes.len(), 0)?;
        let instance = self
            .instances
            .iter()
            .find(|i| i.alive && !i.terminating && i.object == object)
            .ok_or_else(|| refusal(91, "stale or foreign project object"))?;
        let class = self
            .classes
            .iter()
            .find(|c| c.source == instance.source)
            .ok_or_else(|| refusal(91, "unknown project class"))?;
        self.charge(class.fields.len(), 0)?;
        let index = class
            .fields
            .iter()
            .position(|f| f.public && f.name.eq_ignore_ascii_case(field))
            .ok_or_else(|| refusal(438, "public project field not found"))?;
        let location = instance
            .fields
            .get(index)
            .ok_or_else(|| refusal(91, "stale project field"))?;
        let value = self
            .store
            .read(*location)
            .map_err(|_| refusal(91, "stale project field"))?;
        self.charge(0, value.payload_bytes())?;
        Ok(value.clone())
    }
}
