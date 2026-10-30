//! Project-owned instances. Fields are aliases into the ordinary canonical store.
use super::*;

pub(super) struct Delivery {
    pub receiver: vba_values::ObjectRef,
    pub procedure: ir::ProcedureId,
    pub arguments: Vec<Operand>,
}

pub(super) struct Instance {
    pub alive: bool,
    pub terminating: bool,
    pub object: vba_values::ObjectRef,
    pub source: ir::SourceIndex,
    pub document_owner: Option<vba_values::ObjectRef>,
    pub ty: vba_values::NominalType,
    pub fields: Vec<Location>,
}
#[derive(Clone, Copy)]
pub(super) enum ClassReturn {
    Event,
    Initialize(vba_values::ObjectRef),
    Terminate(vba_values::ObjectRef),
    Invoke {
        kind: ir::ForeignKind,
        reference: Option<Location>,
    },
}
#[derive(Clone)]
pub(super) struct Snapshot {
    connections: Vec<u64>,
    object: vba_values::ObjectRef,
    source: ir::SourceIndex,
    document_owner: Option<vba_values::ObjectRef>,
    values: Vec<Value>,
}
pub(super) struct Restore {
    connections: Vec<Vec<u64>>,
    pub instances: Vec<Instance>,
    values: Vec<Vec<Value>>,
    pub mapping: BTreeMap<(u64, u64), vba_values::ObjectRef>,
}
pub(super) fn remap(
    value: Value,
    mapping: &BTreeMap<(u64, u64), vba_values::ObjectRef>,
) -> Result<Value> {
    match value {
        Value::Object(object) if object.host == 0 => Ok(Value::Object(
            *mapping
                .get(&(object.object, object.generation))
                .ok_or_else(|| {
                    runtime_diagnostic(runtime::Error::Vba {
                        number: 91,
                        message: "stale saved project instance".into(),
                    })
                })?,
        )),
        Value::Array(mut array) => {
            for (index, value) in array.elements().to_vec().into_iter().enumerate() {
                array.set(index, remap(value, mapping)?)?;
            }
            Ok(Value::Array(array))
        }
        value => Ok(value),
    }
}
impl Session {
    pub(super) fn raise_event(
        &mut self,
        procedure: ir::ProcedureId,
        arguments: Vec<Operand>,
    ) -> Result<()> {
        let publisher = self
            .top()?
            .receiver
            .ok_or_else(|| invariant("event publisher"))?;
        let event = self
            .program
            .program()
            .procedures
            .get(procedure.0 as usize)
            .ok_or_else(|| invariant("event signature"))?;
        let event_name = event
            .name
            .strip_prefix("Event ")
            .ok_or_else(|| invariant("event name"))?;
        let mut deliveries = Vec::new();
        let mut work = 0u64;
        for instance in self.instances.iter().filter(|i| i.alive && !i.terminating) {
            work = work
                .checked_add(
                    u64::try_from(self.program.program().classes.len()).map_err(|_| resource())?,
                )
                .ok_or_else(resource)?;
            if work
                > self
                    .limits
                    .instructions
                    .saturating_sub(self.counters.instance_work)
            {
                return Err(resource());
            }
            let class = self
                .program
                .program()
                .classes
                .iter()
                .find(|c| c.source == instance.source)
                .ok_or_else(|| invariant("event sink class"))?;
            for (field, location) in class.fields.iter().zip(&instance.fields) {
                work = work.checked_add(1).ok_or_else(resource)?;
                if work
                    > self
                        .limits
                        .instructions
                        .saturating_sub(self.counters.instance_work)
                {
                    return Err(resource());
                }
                if !field.with_events || self.store.read(*location)? != &Value::Object(publisher) {
                    continue;
                }
                let handler_name = format!("{}_{}", field.name, event_name);
                work = work
                    .checked_add(u64::try_from(class.methods.len()).map_err(|_| resource())?)
                    .ok_or_else(resource)?;
                if work
                    > self
                        .limits
                        .instructions
                        .saturating_sub(self.counters.instance_work)
                {
                    return Err(resource());
                }
                let Some(handler) = class.methods.iter().find(|m| {
                    m.name.eq_ignore_ascii_case(&handler_name) && m.kind == ir::ForeignKind::Call
                }) else {
                    continue;
                };
                if deliveries.len() >= self.limits.callbacks.min(256) {
                    return Err(resource());
                }
                deliveries.try_reserve(1).map_err(|_| resource())?;
                deliveries.push((
                    *self
                        .event_connections
                        .get(location)
                        .ok_or_else(|| invariant("event connection"))?,
                    instance.object,
                    handler.procedure,
                ));
            }
        }
        self.counters.instance_work = self
            .counters
            .instance_work
            .checked_add(work)
            .ok_or_else(resource)?;
        if self.counters.instance_work > self.limits.instructions {
            return Err(resource());
        }
        // Capture the connection list once; handlers execute synchronously as ordinary frames.
        // Reference operands alias the original locations so each sink sees previous writes.
        deliveries.sort_by_key(|(order, _, _)| *order);
        let argument_bytes = arguments.iter().try_fold(256usize, |bytes, operand| {
            bytes
                .checked_add(match operand {
                    Operand::Value(value) => 64 + value_bytes(value),
                    Operand::Reference(_) => 64,
                })
                .ok_or_else(resource)
        })?;
        self.charge(
            deliveries
                .len()
                .checked_mul(argument_bytes)
                .ok_or_else(resource)?,
        )?;
        let mut pending = VecDeque::new();
        pending
            .try_reserve(deliveries.len())
            .map_err(|_| resource())?;
        for (_, receiver, procedure) in deliveries {
            pending.push_back(Delivery {
                receiver,
                procedure,
                arguments: arguments.clone(),
            });
        }
        self.top_mut()?.events = pending;
        Ok(())
    }
    pub(super) fn release_native_state(&mut self) -> Result<()> {
        for instance in &self.instances {
            if instance.alive {
                for location in &instance.fields {
                    self.store.invalidate(*location)?;
                    self.kinds.remove(location);
                    self.nominal_types.remove(location);
                    self.array_types.remove(location);
                }
            }
        }
        self.instances.clear();
        self.event_connections.clear();
        self.next_connection = 0;
        self.default_instances.clear();
        self.next_receiver = None;
        self.pending_result = None;
        Ok(())
    }
    pub(super) fn snapshot_instances(&self) -> Result<Vec<Snapshot>> {
        self.instances
            .iter()
            .filter(|i| i.alive)
            .map(|i| {
                Ok(Snapshot {
                    connections: i
                        .fields
                        .iter()
                        .map(|l| self.event_connections.get(l).copied().unwrap_or(0))
                        .collect(),
                    object: i.object,
                    source: i.source,
                    document_owner: i.document_owner,
                    values: i
                        .fields
                        .iter()
                        .map(|l| self.store.read(*l).cloned())
                        .collect::<Result<Vec<_>>>()?,
                })
            })
            .collect()
    }
    pub(super) fn prepare_instances(&mut self, saved: Vec<Snapshot>) -> Result<Restore> {
        if saved.len() > 1024 || !self.instances.is_empty() {
            return Err(command("instance restore requires fresh bounded heap"));
        }
        self.charge(saved.len() * 512)?;
        let mut mapping = BTreeMap::new();
        let mut instances = Vec::new();
        let mut values = Vec::new();
        let mut connections = Vec::new();
        reserve(&mut instances, saved.len())?;
        reserve(&mut values, saved.len())?;
        for (index, snapshot) in saved.iter().enumerate() {
            let class = self
                .program
                .program()
                .classes
                .iter()
                .find(|c| c.source == snapshot.source)
                .ok_or_else(|| command("saved class not found"))?;
            if snapshot.document_owner.is_some_and(|owner| {
                !class.document
                    || owner.host == 0
                    || self
                        .binding
                        .as_ref()
                        .is_none_or(|binding| owner.host != binding.identity)
            }) {
                return Err(command(
                    "saved document owner does not match the current host",
                ));
            }
            if snapshot.values.len() != class.fields.len() {
                return Err(command("saved class fields differ"));
            }
            if snapshot.connections.len() != class.fields.len() {
                return Err(command("saved event fields differ"));
            }
            connections.push(snapshot.connections.clone());
            let object = vba_values::ObjectRef {
                host: 0,
                object: u64::try_from(index).map_err(|_| resource())? + 1,
                generation: u64::from(self.instance.0),
            };
            if mapping
                .insert((snapshot.object.object, snapshot.object.generation), object)
                .is_some()
            {
                return Err(command("duplicate saved instance"));
            }
            instances.push(Instance {
                alive: true,
                terminating: false,
                object,
                source: class.source,
                document_owner: snapshot.document_owner,
                ty: vba_values::NominalType::new("VBAProject", "1", &class.name)?,
                fields: vec![],
            });
        }
        for (snapshot, instance) in saved.into_iter().zip(&instances) {
            let class = self
                .program
                .program()
                .classes
                .iter()
                .find(|c| c.source == instance.source)
                .ok_or_else(|| invariant("restored class"))?;
            let mut fields = Vec::new();
            reserve(&mut fields, snapshot.values.len())?;
            let mut bytes = 0usize;
            for (value, field) in snapshot.values.into_iter().zip(&class.fields) {
                bytes = bytes
                    .checked_add(value_bytes(&value) + 256)
                    .ok_or_else(resource)?;
                let value = remap(value, &mapping)?;
                check_array_identity(field.nominal.as_ref(), field.array, &value)?;
                if field
                    .array
                    .is_some_and(|kind| !matches!(&value,Value::Array(array) if array.kind()==kind))
                {
                    return Err(command("saved field array kind"));
                }
                let value = runtime::bind(field.kind, &value).map_err(runtime_diagnostic)?;
                check_nominal(
                    field.nominal.as_ref(),
                    &value,
                    (
                        &instances,
                        self.instance,
                        !self.program.program().classes.is_empty(),
                    ),
                    self.type_resolver.as_deref(),
                    self.binding.as_ref().map(|b| b.identity),
                    &mut self.counters.type_checks,
                    self.limits.instructions,
                )?;
                fields.push(value);
            }
            self.charge(bytes)?;
            values.push(fields);
        }
        Ok(Restore {
            connections,
            instances,
            values,
            mapping,
        })
    }
    pub(super) fn install_instances(&mut self, mut restored: Restore) -> Result<()> {
        for ((instance, values), connections) in restored
            .instances
            .iter_mut()
            .zip(restored.values)
            .zip(restored.connections)
        {
            let fields = self
                .program
                .program()
                .classes
                .iter()
                .find(|c| c.source == instance.source)
                .ok_or_else(|| invariant("installed class"))?
                .fields
                .clone();
            reserve(&mut instance.fields, fields.len())?;
            for ((field, value), connection) in fields.into_iter().zip(values).zip(connections) {
                let location = self.store.allocate(value)?;
                self.allocated_locations = self
                    .allocated_locations
                    .checked_add(1)
                    .ok_or_else(resource)?;
                self.kinds.insert(location, field.kind);
                if let Some(kind) = field.array {
                    self.array_types.insert(location, kind);
                }
                if let Some(ty) = field.nominal {
                    self.nominal_types.insert(location, ty);
                }
                instance.fields.push(location);
                if field.with_events {
                    self.event_connections.insert(location, connection);
                    self.next_connection = self.next_connection.max(connection);
                }
            }
        }
        self.instances = restored.instances;
        Ok(())
    }
    pub(super) fn instance_object(&self, object: vba_values::ObjectRef) -> Result<&Instance> {
        object
            .object
            .checked_sub(1)
            .and_then(|index| usize::try_from(index).ok())
            .and_then(|index| self.instances.get(index))
            .filter(|i| i.alive && i.object == object)
            .ok_or_else(|| {
                runtime_diagnostic(runtime::Error::Vba {
                    number: 91,
                    message: "stale project instance".into(),
                })
            })
    }
    pub(super) fn bind_document_owner(
        &mut self,
        object: vba_values::ObjectRef,
        owner: vba_values::ObjectRef,
    ) -> Result<()> {
        let instance = self.instance_object(object)?;
        let class = self
            .program
            .program()
            .classes
            .iter()
            .find(|class| class.source == instance.source)
            .ok_or_else(|| invariant("document owner class"))?;
        let identity = self.binding.as_ref().map(|binding| binding.identity);
        if !class.document || owner.host == 0 || identity != Some(owner.host) {
            return Err(command("document owner must be a current host object"));
        }
        if instance
            .document_owner
            .is_some_and(|current| current != owner)
        {
            return Err(command(
                "document module is already bound to another host object",
            ));
        }
        let instance = self
            .instances
            .iter_mut()
            .find(|instance| instance.object == object && instance.alive)
            .ok_or_else(|| invariant("document owner instance"))?;
        instance.document_owner = Some(owner);
        Ok(())
    }
    pub(super) fn document_forward_target(
        &self,
        object: vba_values::ObjectRef,
        site: &ir::ForeignSite,
    ) -> Result<Option<vba_values::ObjectRef>> {
        let instance = self.instance_object(object)?;
        let class = self
            .program
            .program()
            .classes
            .iter()
            .find(|class| class.source == instance.source)
            .ok_or_else(|| invariant("document forwarding class"))?;
        if !class.document {
            return Ok(None);
        }
        let member = if site.member.is_empty() {
            class.default_member.as_deref().unwrap_or("")
        } else {
            site.member.as_str()
        };
        if class
            .fields
            .iter()
            .any(|field| field.name.eq_ignore_ascii_case(member))
            || class
                .methods
                .iter()
                .any(|method| method.name.eq_ignore_ascii_case(member))
        {
            return Ok(None);
        }
        Ok(instance.document_owner)
    }
    pub(super) fn new_instance(&mut self, source: ir::SourceIndex) -> Result<()> {
        let object = self.allocate_instance(source)?;
        self.push(Operand::Value(Value::Object(object)))?;
        self.advance()?;
        self.initialize_instance(source, object, true)
    }
    pub(super) fn allocate_instance(
        &mut self,
        source: ir::SourceIndex,
    ) -> Result<vba_values::ObjectRef> {
        let class = self
            .program
            .program()
            .classes
            .iter()
            .find(|c| c.source == source)
            .cloned()
            .ok_or_else(|| invariant("class source"))?;
        if self.instances.len() >= 1024
            || self
                .allocated_locations
                .checked_add(class.fields.len())
                .is_none_or(|n| n > self.limits.locations.min(4096))
        {
            return Err(resource());
        }
        self.charge(256 + class.fields.len() * 256)?;
        let mut values = Vec::new();
        reserve(&mut values, class.fields.len())?;
        for field in &class.fields {
            let value = if let Some(kind) = field.array {
                Value::Array(
                    vba_values::VbaArray::new(kind, field.bounds.clone(), field.bounds.is_empty())?
                        .with_nominal_type(field.nominal.clone())?,
                )
            } else {
                field.kind.initial_value()
            };
            self.charge(value_bytes(&value))?;
            values.push(value);
        }
        let mut fields = Vec::new();
        reserve(&mut fields, class.fields.len())?;
        reserve(&mut self.instances, 1)?;
        for (field, value) in class.fields.iter().zip(values) {
            let location = self.store.allocate(value)?;
            self.allocated_locations = self
                .allocated_locations
                .checked_add(1)
                .ok_or_else(resource)?;
            self.kinds.insert(location, field.kind);
            if let Some(kind) = field.array {
                self.array_types.insert(location, kind);
            }
            if let Some(ty) = &field.nominal {
                self.nominal_types.insert(location, ty.clone());
            }
            fields.push(location);
            if field.with_events {
                self.event_connections.insert(location, 0);
            }
        }
        let object = vba_values::ObjectRef {
            host: 0,
            object: u64::try_from(self.instances.len()).map_err(|_| resource())? + 1,
            generation: u64::from(self.instance.0),
        };
        self.instances.push(Instance {
            alive: true,
            terminating: false,
            object,
            source,
            document_owner: None,
            ty: vba_values::NominalType::new("VBAProject", "1", &class.name)?,
            fields,
        });
        Ok(object)
    }
    pub(super) fn initialize_instance(
        &mut self,
        source: ir::SourceIndex,
        object: vba_values::ObjectRef,
        lifecycle_return: bool,
    ) -> Result<()> {
        let class = self
            .program
            .program()
            .classes
            .iter()
            .find(|c| c.source == source)
            .cloned()
            .ok_or_else(|| invariant("class source"))?;
        if let Some(initializer) = class
            .methods
            .iter()
            .find(|m| m.name.eq_ignore_ascii_case("Class_Initialize"))
        {
            if !self
                .program
                .program()
                .procedures
                .get(initializer.procedure.0 as usize)
                .is_some_and(|p| p.parameters.is_empty() && p.kind == ir::ProcedureKind::Sub)
            {
                return Err(invariant("initializer signature"));
            }
            self.next_receiver = Some(object);
            self.push_frame(initializer.procedure, vec![])?;
            if lifecycle_return {
                self.top_mut()?.class_return = Some(ClassReturn::Initialize(object));
            }
        }
        Ok(())
    }
    pub(super) fn finish_class_call(
        &mut self,
        kind: ir::ForeignKind,
        reference: Option<Location>,
        value: Value,
    ) -> Result<()> {
        if matches!(kind, ir::ForeignKind::ValueGet) {
            self.push_prepaid(value)
        } else if matches!(kind, ir::ForeignKind::ArgumentGet) {
            self.host_result_reference = reference;
            self.accept_host(foreign::Outcome::Complete(value))
        } else {
            self.push_prepaid(value)?;
            self.advance()
        }
    }
    pub(super) fn invoke_class(
        &mut self,
        object: vba_values::ObjectRef,
        site: ir::ForeignSite,
        args: Vec<foreign::Argument>,
    ) -> Result<()> {
        let instance = self.instance_object(object)?;
        let class = self
            .program
            .program()
            .classes
            .iter()
            .find(|c| c.source == instance.source)
            .cloned()
            .ok_or_else(|| invariant("instance class"))?;
        let fields = instance.fields.clone();
        self.counters.instance_work = self
            .counters
            .instance_work
            .checked_add(
                u64::try_from(class.fields.len() + class.methods.len() + 1)
                    .map_err(|_| resource())?,
            )
            .ok_or_else(resource)?;
        if self.counters.instance_work > self.limits.instructions {
            return Err(resource());
        }
        self.charge((class.fields.len() + class.methods.len() + 1) * 256)?;
        let inside = self.proc()?.source == class.source;
        let kind = match site.kind {
            ir::ForeignKind::ValueGet
            | ir::ForeignKind::ArgumentGet
            | ir::ForeignKind::DefaultGet => ir::ForeignKind::Get,
            ir::ForeignKind::DefaultLet | ir::ForeignKind::AssignValue => ir::ForeignKind::Let,
            ir::ForeignKind::DefaultSet => ir::ForeignKind::Set,
            kind => kind,
        };
        let member = if site.member.is_empty() {
            class.default_member.as_deref().ok_or_else(|| {
                runtime_diagnostic(runtime::Error::Vba {
                    number: 438,
                    message: "class has no declared default member".into(),
                })
            })?
        } else {
            site.member.as_str()
        };
        if let Some((index, field)) = class
            .fields
            .iter()
            .enumerate()
            .find(|(_, f)| f.name.eq_ignore_ascii_case(member) && (f.public || inside))
        {
            let location = *fields.get(index).ok_or_else(|| invariant("field index"))?;
            let value = if matches!(kind, ir::ForeignKind::Call | ir::ForeignKind::Get) {
                if !args.is_empty() {
                    return Err(runtime_diagnostic(runtime::Error::Vba {
                        number: 450,
                        message: "field takes no index arguments".into(),
                    }));
                }
                self.read_value(location)?
            } else if matches!(kind, ir::ForeignKind::Let | ir::ForeignKind::Set) && args.len() == 1
            {
                let argument = args.first().ok_or_else(|| invariant("field argument"))?;
                let value = self.class_argument(&argument.value)?;
                let value = runtime::assign(field.kind, &value, kind == ir::ForeignKind::Set)
                    .map_err(runtime_diagnostic)?;
                self.write_value(location, value)?;
                Value::Empty
            } else {
                return Err(runtime_diagnostic(runtime::Error::Vba {
                    number: 450,
                    message: "invalid field application".into(),
                }));
            };
            if site.kind == ir::ForeignKind::ArgumentGet {
                let scratch = site
                    .result_reference
                    .ok_or_else(|| invariant("field reference scratch"))?;
                let expected = self.location(scratch)?;
                if self.kinds.get(&expected).is_some_and(|kind| {
                    *kind != ScalarKind::Variant && Some(kind) != self.kinds.get(&location)
                }) {
                    return Err(runtime_diagnostic(runtime::Error::Vba {
                        number: 13,
                        message: "ByRef field type mismatch".into(),
                    }));
                }
                self.push(Operand::Reference(location))?;
                self.advance()?;
                return Ok(());
            }
            return self.finish_class_call(
                site.kind,
                site.result_reference
                    .map(|s| self.location(s))
                    .transpose()?,
                value,
            );
        }
        let method = class
            .methods
            .iter()
            .find(|m| {
                m.name.eq_ignore_ascii_case(member)
                    && (m.kind == kind
                        || kind == ir::ForeignKind::Call && m.kind == ir::ForeignKind::Get)
                    && (m.public || inside)
            })
            .ok_or_else(|| {
                runtime_diagnostic(runtime::Error::Vba {
                    number: 438,
                    message: "class member is inaccessible or unsupported".into(),
                })
            })?;
        let procedure = method.procedure;
        let params = self
            .program
            .program()
            .procedures
            .get(procedure.0 as usize)
            .ok_or_else(|| invariant("class procedure"))?
            .parameters
            .clone();
        let mut mapped = Vec::new();
        reserve(&mut mapped, params.len())?;
        mapped.resize_with(params.len(), || None);
        for (position, arg) in args.into_iter().enumerate() {
            let index = if let Some(name) = arg.name {
                if name == "$value" {
                    params
                        .len()
                        .checked_sub(1)
                        .ok_or_else(|| invariant("property value"))?
                } else {
                    params
                        .iter()
                        .position(|p| p.name.eq_ignore_ascii_case(&name))
                        .ok_or_else(|| {
                            runtime_diagnostic(runtime::Error::Vba {
                                number: 448,
                                message: "named argument not found".into(),
                            })
                        })?
                }
            } else {
                position
            };
            let slot = mapped.get_mut(index).ok_or_else(|| {
                runtime_diagnostic(runtime::Error::Vba {
                    number: 450,
                    message: "wrong number of arguments".into(),
                })
            })?;
            if slot.is_some() {
                return Err(runtime_diagnostic(runtime::Error::Vba {
                    number: 450,
                    message: "duplicate argument".into(),
                }));
            }
            *slot = Some(arg.value);
        }
        let mut arguments = Vec::new();
        reserve(&mut arguments, params.len())?;
        for (param, arg) in params.iter().zip(mapped) {
            let arg = arg.filter(|a| !matches!(a, foreign::ArgumentValue::Omitted));
            let value = match arg {
                Some(foreign::ArgumentValue::ByRef(location))
                    if param.mode == ir::ParameterMode::ByRef =>
                {
                    Operand::Reference(location)
                }
                Some(value) => Operand::Value(self.class_argument(&value)?),
                None if !param.required => {
                    Operand::Value(param.default.clone().unwrap_or(Value::Missing))
                }
                None => {
                    return Err(runtime_diagnostic(runtime::Error::Vba {
                        number: 449,
                        message: "argument not optional".into(),
                    }));
                }
            };
            arguments.push(value);
        }
        let reference = site
            .result_reference
            .map(|s| self.location(s))
            .transpose()?;
        self.next_receiver = Some(object);
        self.push_frame(procedure, arguments)?;
        self.top_mut()?.class_return = Some(ClassReturn::Invoke {
            kind: site.kind,
            reference,
        });
        Ok(())
    }
    fn class_argument(&self, arg: &foreign::ArgumentValue) -> Result<Value> {
        match arg {
            foreign::ArgumentValue::ByVal(value) => Ok(value.clone()),
            foreign::ArgumentValue::ByRef(location) => Ok(self.store.read(*location)?.clone()),
            foreign::ArgumentValue::Omitted => Ok(Value::Missing),
        }
    }

    pub(super) fn retire_instance(&mut self, object: vba_values::ObjectRef) -> Result<()> {
        let instance = self
            .instances
            .iter_mut()
            .find(|i| i.object == object)
            .ok_or_else(|| invariant("retired instance"))?;
        instance.alive = false;
        let fields = std::mem::take(&mut instance.fields);
        for field in fields {
            self.event_connections.remove(&field);
            self.store.invalidate(field)?;
            self.kinds.remove(&field);
            self.nominal_types.remove(&field);
            self.array_types.remove(&field);
        }
        Ok(())
    }
    /// Reference-count behavior: fields of every live instance retain references,
    /// so cycles remain alive. No code runs during abnormal/reset cleanup.
    pub(super) fn settle_instances(&mut self) -> Result<bool> {
        if self.instances.is_empty() {
            return Ok(false);
        }
        loop {
            let mut roots = BTreeSet::new();
            let mut work = u64::try_from(self.instances.len()).map_err(|_| resource())?;
            let mut visit = |value: &Value| -> Result<()> {
                let values = if let Value::Array(array) = value {
                    array.elements()
                } else {
                    std::slice::from_ref(value)
                };
                work = work
                    .checked_add(u64::try_from(values.len()).map_err(|_| resource())?)
                    .ok_or_else(resource)?;
                for value in values {
                    if let Value::Object(object) = value
                        && object.host == 0
                    {
                        roots.insert((object.object, object.generation));
                    }
                }
                Ok(())
            };
            for frame in &self.frames {
                for delivery in &frame.events {
                    visit(&Value::Object(delivery.receiver))?;
                    for operand in &delivery.arguments {
                        match operand {
                            Operand::Value(value) => visit(value)?,
                            Operand::Reference(location) => visit(self.store.read(*location)?)?,
                        }
                    }
                }
                for location in &frame.slots {
                    visit(self.store.read(*location)?)?;
                }
                for operand in &frame.stack {
                    match operand {
                        Operand::Value(value) => visit(value)?,
                        Operand::Reference(location) => visit(self.store.read(*location)?)?,
                    }
                }
            }
            for object in self.default_instances.values() {
                visit(&Value::Object(*object))?;
            }
            for callback in &self.callbacks {
                if let Some(object) = callback.receiver {
                    visit(&Value::Object(object))?;
                }
                for value in &callback.arguments {
                    visit(value)?;
                }
            }
            for location in self.persistent.values() {
                visit(self.store.read(*location)?)?;
            }
            for instance in self.instances.iter().filter(|i| i.alive) {
                for location in &instance.fields {
                    visit(self.store.read(*location)?)?;
                }
            }
            if let Some(ReturnValue::Function(value)) = &self.pending_result {
                visit(value)?;
            }
            self.counters.instance_work = self
                .counters
                .instance_work
                .checked_add(work)
                .ok_or_else(resource)?;
            if self.counters.instance_work > self.limits.instructions {
                return Err(resource());
            }
            let candidate = self
                .instances
                .iter()
                .find(|i| {
                    i.alive
                        && !i.terminating
                        && !roots.contains(&(i.object.object, i.object.generation))
                })
                .map(|i| (i.object, i.source));
            let Some((object, source)) = candidate else {
                return Ok(false);
            };
            let terminator = self
                .program
                .program()
                .classes
                .iter()
                .find(|c| c.source == source)
                .and_then(|c| {
                    c.methods
                        .iter()
                        .find(|m| m.name.eq_ignore_ascii_case("Class_Terminate"))
                })
                .map(|m| m.procedure);
            if let Some(procedure) = terminator {
                self.instances
                    .iter_mut()
                    .find(|i| i.object == object)
                    .ok_or_else(|| invariant("terminating instance"))?
                    .terminating = true;
                self.next_receiver = Some(object);
                self.push_frame(procedure, vec![])?;
                self.top_mut()?.class_return = Some(ClassReturn::Terminate(object));
                return Ok(true);
            }
            self.retire_instance(object)?;
        }
    }
}
