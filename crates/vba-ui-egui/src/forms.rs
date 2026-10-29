//! Reusable form designer and real runtime controls. All edits/events are explicit IDE commands.
use super::*;
use vba_ide::forms::{Align, Bounds, ControlKind, DesignEdit, SameSize};
#[derive(Clone, Debug)]
pub enum FormCommand {
    Add,
    Edit(SourceId, DesignEdit),
    Select(SourceId, Option<u32>, bool),
    SelectAll(SourceId),
    Tab(SourceId, bool),
    Grid(SourceId, f32, bool),
    Zoom(SourceId, u16),
    History(SourceId, bool),
    Handler(SourceId, Option<u32>, String),
    Run(SourceId, bool),
    Event(SourceId, Option<u32>, Option<String>),
}
pub(super) fn apply_form(
    ide: &mut Ide,
    command: FormCommand,
    host: &mut impl vba_ide::Host,
) -> vba_ide::Result<()> {
    match command {
        FormCommand::Add => {
            for n in 1..=vba_ide::MAX_MODULES {
                let name = format!("UserForm{n}");
                if !ide
                    .modules
                    .iter()
                    .any(|m| m.name.eq_ignore_ascii_case(&format!("{name}.frm")))
                {
                    ide.add_form(&name)?;
                    return Ok(());
                }
            }
            Err(vba_ide::Diagnostic::new("VBA_IDE", "form module budget"))
        }
        FormCommand::Edit(id, edit) => ide.edit_form(id, edit),
        FormCommand::Select(id, c, extend) => ide.select_form_control(id, c, extend),
        FormCommand::SelectAll(id) => {
            let ids = ide
                .form(id)
                .map(|d| d.form().controls.iter().map(|c| c.id).collect::<Vec<_>>())
                .unwrap_or_default();
            ide.select_form_control(id, None, false)?;
            for c in ids {
                if ide.form(id).is_some_and(|d| !d.selection().contains(&c)) {
                    ide.select_form_control(id, Some(c), true)?;
                }
            }
            Ok(())
        }
        FormCommand::Tab(id, backward) => ide.form_tab(id, backward),
        FormCommand::Grid(id, grid, snap) => ide.form_grid(id, grid, snap),
        FormCommand::Zoom(id, zoom) => ide.form_zoom(id, zoom),
        FormCommand::History(id, redo) => ide.form_history(id, redo),
        FormCommand::Handler(id, c, event) => ide.generate_form_handler(id, c, &event),
        FormCommand::Run(id, modal) => ide.run_form(id, modal, host),
        FormCommand::Event(id, c, text) => ide.form_event(id, c, text, host),
    }
}
#[derive(Clone, Debug)]
pub struct FormLayoutReport {
    pub source: SourceId,
    pub canvas: egui::Rect,
    pub controls: Vec<(u32, egui::Rect)>,
    pub handles: Vec<(u32, egui::Rect)>,
}
fn control_text(value: vba_ide::Value) -> String {
    match value {
        vba_ide::Value::String(text) => String::from_utf16_lossy(text.units()),
        value => vba_ide::display_value(&value),
    }
}
fn command(commands: &mut Vec<Command>, id: SourceId, edit: DesignEdit) {
    commands.push(Command::Form(FormCommand::Edit(id, edit)));
}
fn string_property(ui: &mut egui::Ui, label: &str, value: &mut String, limit: usize) -> bool {
    let label = ui.label(label);
    let changed = ui
        .add(TextEdit::singleline(value).char_limit(limit))
        .labelled_by(label.id)
        .changed();
    ui.end_row();
    changed
}
fn number_property(ui: &mut egui::Ui, label: &str, value: &mut f32) -> bool {
    let label = ui.label(label);
    let changed = ui
        .add(
            egui::DragValue::new(value)
                .speed(1.0)
                .range(-32767.0..=32767.0),
        )
        .labelled_by(label.id)
        .changed();
    ui.end_row();
    changed
}
impl EditorUi {
    pub(super) fn form_format(
        &self,
        ui: &mut egui::Ui,
        ide: &Ide,
        commands: &mut Vec<Command>,
    ) -> bool {
        let Some(id) = ide
            .active_module()
            .map(|m| m.id)
            .filter(|id| self.form_view && ide.form(*id).is_some())
        else {
            return false;
        };
        ui.add_enabled_ui(ide.mode() == Mode::Design, |ui| {
            for (label, edit) in [
                ("Align Left", DesignEdit::Align(Align::Left)),
                ("Align Centers", DesignEdit::Align(Align::Center)),
                ("Align Right", DesignEdit::Align(Align::Right)),
                ("Align Top", DesignEdit::Align(Align::Top)),
                ("Align Middles", DesignEdit::Align(Align::Middle)),
                ("Align Bottom", DesignEdit::Align(Align::Bottom)),
                ("Align to Grid", DesignEdit::Align(Align::Grid)),
                ("Same Width", DesignEdit::SameSize(SameSize::Width)),
                ("Same Height", DesignEdit::SameSize(SameSize::Height)),
                ("Same Size", DesignEdit::SameSize(SameSize::Both)),
                ("Space Horizontally", DesignEdit::Space { horizontal: true }),
                ("Space Vertically", DesignEdit::Space { horizontal: false }),
                (
                    "Center Horizontally",
                    DesignEdit::Center { horizontal: true },
                ),
                (
                    "Center Vertically",
                    DesignEdit::Center { horizontal: false },
                ),
                ("Group", DesignEdit::Group),
                ("Ungroup", DesignEdit::Ungroup),
                ("Bring to Front", DesignEdit::Front),
                ("Send to Back", DesignEdit::Back),
            ] {
                if ui.button(label).clicked() {
                    command(commands, id, edit);
                    ui.close();
                }
            }
        });
        true
    }
    pub(super) fn form_designer(
        &mut self,
        ui: &mut egui::Ui,
        ide: &Ide,
        commands: &mut Vec<Command>,
        theme: Theme,
    ) {
        let Some(id) = ide.active_module().map(|m| m.id) else {
            return;
        };
        let Some(designer) = ide.form(id) else {
            return;
        };
        let form = designer.form();
        let editable = ide.mode() == Mode::Design;
        ui.horizontal_wrapped(|ui| {
            ui.label(
                RichText::new(format!(
                    "{} — Design{}",
                    form.name,
                    if designer.dirty() { " *" } else { "" }
                ))
                .strong(),
            );
            if ui.button("View Form Code").clicked() {
                self.form_view = false;
                self.focus_editor = true;
            }
            if ui
                .add_enabled(editable, egui::Button::new("Run Form"))
                .clicked()
            {
                commands.push(Command::Form(FormCommand::Run(id, self.form_modal)));
            }
            ui.checkbox(&mut self.form_modal, "Modal");
            if ui
                .add_enabled(ide.enabled(Action::Reset), egui::Button::new("Reset Form"))
                .clicked()
            {
                commands.push(Command::Action(Action::Reset));
            }
        });
        ui.add_enabled_ui(editable, |ui| {
            ui.horizontal_wrapped(|ui| {
                ui.label("Toolbox:");
                for kind in [
                    ControlKind::Label,
                    ControlKind::TextBox,
                    ControlKind::CommandButton,
                ] {
                    if ui.button(format!("Add {}", kind.name())).clicked() {
                        let offset = (form.controls.len() % 8) as f32 * 12.0;
                        command(
                            commands,
                            id,
                            DesignEdit::Add(
                                kind,
                                Bounds {
                                    left: 12.0 + offset,
                                    top: 12.0 + offset,
                                    width: 90.0,
                                    height: 24.0,
                                },
                            ),
                        );
                    }
                }
                if ui
                    .add_enabled(designer.can_undo(), egui::Button::new("Undo Design"))
                    .clicked()
                {
                    commands.push(Command::Form(FormCommand::History(id, false)));
                }
                if ui
                    .add_enabled(designer.can_redo(), egui::Button::new("Redo Design"))
                    .clicked()
                {
                    commands.push(Command::Form(FormCommand::History(id, true)));
                }
                let mut zoom = designer.zoom();
                if ui
                    .add(egui::Slider::new(&mut zoom, 10..=400).text("Zoom %"))
                    .changed()
                {
                    commands.push(Command::Form(FormCommand::Zoom(id, zoom)));
                }
                let mut grid = designer.grid();
                let mut snap = designer.snap();
                let changed = ui.checkbox(&mut snap, "Snap to Grid").changed();
                let label = ui.label("Grid points");
                if ui
                    .add(egui::DragValue::new(&mut grid).range(1.0..=72.0))
                    .labelled_by(label.id)
                    .changed()
                    || changed
                {
                    commands.push(Command::Form(FormCommand::Grid(id, grid, snap)));
                }
            });
        });
        if !editable {
            ui.label("Reset execution to edit the design. Runtime values are separate.");
        }
        ui.separator();
        let properties = if ui.available_width() < 650.0 {
            egui::Panel::bottom("form.properties.bottom")
                .default_size(240.0)
                .min_size(150.0)
                .max_size((ui.available_height() * 0.6).max(150.0))
        } else {
            egui::Panel::right("form.properties")
                .default_size(220.0)
                .min_size(165.0)
                .max_size(300.0)
        };
        properties.resizable(true).show(ui, |ui| {
            ui.heading("Form Properties");
            egui::ScrollArea::vertical()
                .id_salt("form.props.scroll")
                .show(ui, |ui| {
                    ui.add_enabled_ui(editable, |ui| {
                        if let Some(control) = designer
                            .primary()
                            .and_then(|selected| form.controls.iter().find(|c| c.id == selected))
                        {
                            let mut c = control.clone();
                            let mut changed = false;
                            ui.label(format!("Selected: {} • {}", c.name, c.kind.name()));
                            egui::Grid::new("control.props")
                                .num_columns(2)
                                .striped(true)
                                .show(ui, |ui| {
                                    changed |= string_property(ui, "Control Name", &mut c.name, 64);
                                    changed |= string_property(ui, "Caption", &mut c.caption, 1024);
                                    if c.kind == ControlKind::TextBox {
                                        changed |= string_property(ui, "Text", &mut c.text, 16384);
                                    }
                                    changed |= number_property(ui, "Left", &mut c.bounds.left);
                                    changed |= number_property(ui, "Top", &mut c.bounds.top);
                                    changed |= number_property(ui, "Width", &mut c.bounds.width);
                                    changed |= number_property(ui, "Height", &mut c.bounds.height);
                                    changed |= ui.checkbox(&mut c.enabled, "Enabled").changed();
                                    ui.end_row();
                                    changed |= ui.checkbox(&mut c.visible, "Visible").changed();
                                    ui.end_row();
                                    changed |= ui.checkbox(&mut c.tab_stop, "Tab Stop").changed();
                                    ui.end_row();
                                });
                            if changed {
                                command(commands, id, DesignEdit::Replace(c));
                            }
                            if ui.button("View Default Event").clicked() {
                                commands.push(Command::Form(FormCommand::Handler(
                                    id,
                                    Some(control.id),
                                    control.kind.default_event().into(),
                                )));
                                self.form_view = false;
                                self.navigate = true;
                            }
                            if ui.button("Delete Selection").clicked() {
                                command(commands, id, DesignEdit::Delete);
                            }
                        } else {
                            let mut caption = form.caption.clone();
                            let mut width = form.width;
                            let mut height = form.height;
                            let mut changed = false;
                            ui.label(&form.name);
                            egui::Grid::new("form.props").num_columns(2).show(ui, |ui| {
                                changed |= string_property(ui, "Form Caption", &mut caption, 1024);
                                changed |= number_property(ui, "Form Width", &mut width);
                                changed |= number_property(ui, "Form Height", &mut height);
                            });
                            if changed {
                                command(
                                    commands,
                                    id,
                                    DesignEdit::FormProperties {
                                        caption,
                                        width,
                                        height,
                                    },
                                );
                            }
                            for event in [
                                "Initialize",
                                "Activate",
                                "Deactivate",
                                "QueryClose",
                                "Terminate",
                            ] {
                                if ui.button(format!("View {event} Event")).clicked() {
                                    commands.push(Command::Form(FormCommand::Handler(
                                        id,
                                        None,
                                        event.into(),
                                    )));
                                    self.form_view = false;
                                    self.navigate = true;
                                }
                            }
                        }
                        ui.separator();
                        ui.heading("Tab Order");
                        for (position, control) in form.tab_order.iter().enumerate() {
                            if let Some(c) = form.controls.iter().find(|c| c.id == *control) {
                                ui.horizontal(|ui| {
                                    if ui
                                        .selectable_label(
                                            designer.selection().contains(control),
                                            format!("{}: {}", position, c.name),
                                        )
                                        .clicked()
                                    {
                                        commands.push(Command::Form(FormCommand::Select(
                                            id,
                                            Some(*control),
                                            false,
                                        )));
                                    }
                                    for (label, offset) in [("Earlier", -1isize), ("Later", 1)] {
                                        let next = position
                                            .checked_add_signed(offset)
                                            .filter(|n| *n < form.tab_order.len());
                                        if ui
                                            .add_enabled(next.is_some(), egui::Button::new(label))
                                            .on_hover_text(format!(
                                                "Move {} {label} in tab order",
                                                c.name
                                            ))
                                            .clicked()
                                            && let Some(next) = next
                                        {
                                            let mut order = form.tab_order.clone();
                                            order.swap(position, next);
                                            command(commands, id, DesignEdit::TabOrder(order));
                                        }
                                    }
                                });
                            }
                        }
                    });
                });
        });
        egui::ScrollArea::both()
            .id_salt("form.canvas.scroll")
            .show(ui, |ui| {
                let scale = 4.0 / 3.0 * designer.zoom() as f32 / 100.0;
                let size = egui::vec2(form.width * scale, form.height * scale);
                let (rect, response) = ui.allocate_exact_size(
                    size.max(egui::vec2(40.0, 40.0)),
                    egui::Sense::click_and_drag(),
                );
                ui.painter().rect_filled(rect, 0.0, theme.chrome);
                ui.painter().rect_stroke(
                    rect,
                    0.0,
                    egui::Stroke::new(1.0, theme.border),
                    egui::StrokeKind::Inside,
                );
                let grid = designer.grid() * scale;
                for axis in 0..2 {
                    for n in 1..=400 {
                        let offset = n as f32 * grid;
                        let extent = if axis == 0 {
                            rect.width()
                        } else {
                            rect.height()
                        };
                        if offset >= extent {
                            break;
                        }
                        let (a, b) = if axis == 0 {
                            (
                                rect.min + egui::vec2(offset, 0.0),
                                rect.min + egui::vec2(offset, rect.height()),
                            )
                        } else {
                            (
                                rect.min + egui::vec2(0.0, offset),
                                rect.min + egui::vec2(rect.width(), offset),
                            )
                        };
                        ui.painter()
                            .line_segment([a, b], egui::Stroke::new(0.3, theme.border));
                    }
                }
                if response.clicked() && editable {
                    commands.push(Command::Form(FormCommand::Select(id, None, false)));
                    response.request_focus();
                }
                let mut report = FormLayoutReport {
                    source: id,
                    canvas: rect,
                    controls: vec![],
                    handles: vec![],
                };
                for c in &form.controls {
                    let bounds = egui::Rect::from_min_size(
                        rect.min + egui::vec2(c.bounds.left, c.bounds.top) * scale,
                        egui::vec2(c.bounds.width, c.bounds.height) * scale,
                    );
                    let label = format!("{} ({})", c.name, c.kind.name());
                    let selected = designer.selection().contains(&c.id);
                    let response = ui
                        .put(
                            bounds,
                            egui::Button::new(if c.kind == ControlKind::TextBox {
                                format!("{}: {}", c.name, c.text)
                            } else {
                                format!("{}: {}", c.name, c.caption)
                            })
                            .selected(selected)
                            .sense(egui::Sense::click_and_drag()),
                        )
                        .on_hover_text(&label);
                    response.widget_info(|| {
                        egui::WidgetInfo::selected(
                            egui::accesskit::Role::Button,
                            editable,
                            selected,
                            &label,
                        )
                    });
                    report.controls.push((c.id, bounds));
                    if (response.clicked() || response.drag_started()) && editable {
                        commands.push(Command::Form(FormCommand::Select(
                            id,
                            Some(c.id),
                            ui.input(|i| i.modifiers.shift),
                        )));
                        response.request_focus();
                    }
                    if editable {
                        self.designer_drag(&response, id, c.id, false, scale, commands);
                    }
                    if response.double_clicked() && editable {
                        commands.push(Command::Form(FormCommand::Handler(
                            id,
                            Some(c.id),
                            c.kind.default_event().into(),
                        )));
                        self.form_view = false;
                        self.navigate = true;
                    }
                    if selected {
                        ui.painter().rect_stroke(
                            bounds,
                            0.0,
                            egui::Stroke::new(2.0, theme.accent),
                            egui::StrokeKind::Outside,
                        );
                        let handle =
                            egui::Rect::from_center_size(bounds.max, egui::vec2(10.0, 10.0));
                        let resize = ui
                            .interact(
                                handle,
                                ui.scope_id().with(("form.resize", c.id)),
                                egui::Sense::drag(),
                            )
                            .on_hover_text(format!("Resize {}", c.name));
                        ui.painter().rect_filled(handle, 0.0, theme.accent);
                        report.handles.push((c.id, handle));
                        if editable {
                            self.designer_drag(&resize, id, c.id, true, scale, commands);
                        }
                    }
                    if response.has_focus() && editable {
                        self.form_keys(ui, id, designer.grid(), commands);
                    }
                }
                if response.has_focus() && editable {
                    self.form_keys(ui, id, designer.grid(), commands);
                }
                self.form_report = Some(report);
            });
    }
    fn designer_drag(
        &mut self,
        response: &egui::Response,
        id: SourceId,
        control: u32,
        resize: bool,
        scale: f32,
        commands: &mut Vec<Command>,
    ) {
        if response.drag_started() {
            self.form_drag = Some((id, control, resize, egui::Vec2::ZERO));
        }
        if let Some((source, target, sizing, delta)) = &mut self.form_drag
            && *source == id
            && *target == control
            && *sizing == resize
        {
            *delta += response.drag_delta() / scale;
            if response.drag_stopped() {
                let delta = *delta;
                self.form_drag = None;
                if delta != egui::Vec2::ZERO {
                    command(
                        commands,
                        id,
                        if resize {
                            DesignEdit::Resize(delta.x, delta.y)
                        } else {
                            DesignEdit::Move(delta.x, delta.y)
                        },
                    );
                }
            }
        }
    }
    fn form_keys(&self, ui: &egui::Ui, id: SourceId, grid: f32, commands: &mut Vec<Command>) {
        if ui.input_mut(|i| i.consume_key(egui::Modifiers::COMMAND, egui::Key::A)) {
            commands.push(Command::Form(FormCommand::SelectAll(id)));
        }
        if ui.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::Delete)) {
            command(commands, id, DesignEdit::Delete);
        }
        for (key, dx, dy) in [
            (egui::Key::ArrowLeft, -grid, 0.0),
            (egui::Key::ArrowRight, grid, 0.0),
            (egui::Key::ArrowUp, 0.0, -grid),
            (egui::Key::ArrowDown, 0.0, grid),
        ] {
            if ui.input_mut(|i| i.consume_key(egui::Modifiers::SHIFT, key)) {
                command(commands, id, DesignEdit::Resize(dx, dy));
            }
            if ui.input_mut(|i| i.consume_key(egui::Modifiers::NONE, key)) {
                command(commands, id, DesignEdit::Move(dx, dy));
            }
        }
        for (modifier, backward) in [
            (egui::Modifiers::SHIFT, true),
            (egui::Modifiers::NONE, false),
        ] {
            if ui.input_mut(|i| i.consume_key(modifier, egui::Key::Tab)) {
                commands.push(Command::Form(FormCommand::Tab(id, backward)));
            }
        }
    }
    pub(super) fn form_windows(
        &mut self,
        ctx: &egui::Context,
        ide: &Ide,
        commands: &mut Vec<Command>,
        theme: Theme,
    ) {
        if matches!(ide.mode(), Mode::Break | Mode::Stale) {
            self.form_runtime_focus.clear();
            self.form_last_focus = None;
            return;
        }
        let last_focus = self.form_last_focus.take();
        let previous_focus = std::mem::take(&mut self.form_runtime_focus);
        for module in &ide.modules {
            let Some(designer) = ide.form(module.id).filter(|_| ide.form_visible(module.id)) else {
                continue;
            };
            let form = designer.form();
            let id = module.id;
            let idle = ide.form_can_deliver(id);
            let mut ordered = previous_focus
                .iter()
                .filter(|(source, _, _)| *source == id)
                .map(|(_, control, target)| {
                    let index = match ide.form_runtime_field(id, Some(*control), "TabIndex") {
                        Ok(vba_ide::Value::Long(n)) => n,
                        _ => i32::MAX,
                    };
                    (index, *control, *target)
                })
                .collect::<Vec<_>>();
            ordered.sort_by_key(|(index, control, _)| (*index, *control));
            if idle
                && let Some(at) = ordered
                    .iter()
                    .position(|(_, _, target)| Some(*target) == last_focus)
            {
                for (modifiers, backward) in [
                    (egui::Modifiers::SHIFT, true),
                    (egui::Modifiers::NONE, false),
                ] {
                    if ctx.input_mut(|i| i.consume_key(modifiers, egui::Key::Tab)) {
                        let next = if backward {
                            (at + ordered.len() - 1) % ordered.len()
                        } else {
                            (at + 1) % ordered.len()
                        };
                        if let Some((_, _, target)) = ordered.get(next) {
                            ctx.memory_mut(|m| {
                                m.move_focus(egui::FocusDirection::None);
                                m.request_focus(*target);
                            });
                        }
                    }
                }
            }
            let mut rendered_focus = Vec::new();
            let show = |ui: &mut egui::Ui| {
                window_theme(ui, theme);
                ui.horizontal(|ui| {
                    ui.heading(
                        ide.form_runtime_field(id, None, "Caption")
                            .map(control_text)
                            .unwrap_or_else(|_| form.caption.clone()),
                    );
                    if ui
                        .add_enabled(idle, egui::Button::new("Close Form"))
                        .clicked()
                    {
                        commands.push(Command::Form(FormCommand::Event(id, None, None)));
                    }
                });
                ui.add_enabled_ui(idle, |ui| {
                    let scale = 4.0 / 3.0;
                    let (rect, _) = ui.allocate_exact_size(
                        egui::vec2(form.width * scale, form.height * scale),
                        egui::Sense::hover(),
                    );
                    let mut focus = Vec::new();
                    for c in &form.controls {
                        if ide.form_runtime_field(id, Some(c.id), "Visible").ok()
                            != Some(vba_ide::Value::Boolean(true))
                        {
                            continue;
                        }
                        let enabled = ide.form_runtime_field(id, Some(c.id), "Enabled").ok()
                            == Some(vba_ide::Value::Boolean(true));
                        let caption = ide
                            .form_runtime_field(id, Some(c.id), "Caption")
                            .map(control_text)
                            .unwrap_or_else(|_| c.caption.clone());
                        let number = |name: &str, fallback: f32| match ide.form_runtime_field(
                            id,
                            Some(c.id),
                            name,
                        ) {
                            Ok(vba_ide::Value::Single(n))
                                if n.is_finite() && n.abs() <= 32767.0 =>
                            {
                                n
                            }
                            _ => fallback,
                        };
                        let bounds = egui::Rect::from_min_size(
                            rect.min
                                + egui::vec2(
                                    number("Left", c.bounds.left),
                                    number("Top", c.bounds.top),
                                ) * scale,
                            egui::vec2(
                                number("Width", c.bounds.width).max(0.0),
                                number("Height", c.bounds.height).max(0.0),
                            ) * scale,
                        );
                        let widget_id = ui.scope_id().with(("runtime.control", c.id));
                        let tab_stop = ide.form_runtime_field(id, Some(c.id), "TabStop").ok()
                            == Some(vba_ide::Value::Boolean(true));
                        let index = match ide.form_runtime_field(id, Some(c.id), "TabIndex") {
                            Ok(vba_ide::Value::Long(n)) => n,
                            _ => i32::MAX,
                        };
                        if enabled && tab_stop {
                            focus.push((index, c.id, widget_id));
                        }
                        ui.push_id(("runtime", c.id), |ui| {
                            ui.add_enabled_ui(enabled, |ui| match c.kind {
                                ControlKind::TextBox => {
                                    let mut text = ide
                                        .form_runtime_field(id, Some(c.id), "VbaText")
                                        .map(control_text)
                                        .unwrap_or_default();
                                    let response = ui.put(
                                        bounds,
                                        TextEdit::singleline(&mut text)
                                            .id(widget_id)
                                            .char_limit(16384)
                                            .hint_text(&c.name),
                                    );
                                    response.widget_info(|| {
                                        egui::WidgetInfo::labeled(
                                            egui::accesskit::Role::TextInput,
                                            enabled,
                                            &c.name,
                                        )
                                    });
                                    if response.changed() {
                                        commands.push(Command::Form(FormCommand::Event(
                                            id,
                                            Some(c.id),
                                            Some(text),
                                        )));
                                    }
                                }
                                ControlKind::Label | ControlKind::CommandButton => {
                                    let response = ui.put(
                                        bounds,
                                        egui::Button::new(caption)
                                            .frame(c.kind == ControlKind::CommandButton),
                                    );
                                    if let Some(target) =
                                        focus.iter_mut().find(|(_, control, _)| *control == c.id)
                                    {
                                        target.2 = response.id;
                                    }
                                    if !tab_stop {
                                        response.surrender_focus();
                                    }
                                    response.widget_info(|| {
                                        egui::WidgetInfo::labeled(
                                            egui::accesskit::Role::Button,
                                            enabled,
                                            &c.name,
                                        )
                                    });
                                    if response.clicked() {
                                        commands.push(Command::Form(FormCommand::Event(
                                            id,
                                            Some(c.id),
                                            None,
                                        )));
                                    }
                                }
                            });
                        });
                    }
                    rendered_focus.extend(
                        focus
                            .into_iter()
                            .map(|(_, control, target)| (id, control, target)),
                    );
                });
                if !idle {
                    ui.label("Execution paused or running — use the debugger to continue.");
                }
                if ui.button("Reset Form Execution").clicked() {
                    commands.push(Command::Action(Action::Reset));
                }
            };
            let scope = self
                .scope
                .unwrap_or(self.instance)
                .with(("form.runtime", id.0));
            if ide.mode() == Mode::Waiting && ide.form_is_modal(id) {
                egui::Modal::new(scope)
                    .frame(
                        egui::Frame::new()
                            .fill(theme.chrome)
                            .stroke(egui::Stroke::new(1.0, theme.border))
                            .corner_radius(5)
                            .inner_margin(8),
                    )
                    .show(ctx, show);
            } else {
                egui::Window::new(format!("{} — Running", form.name))
                    .id(scope)
                    .frame(
                        egui::Frame::new()
                            .fill(theme.chrome)
                            .stroke(egui::Stroke::new(1.0, theme.border))
                            .corner_radius(5)
                            .inner_margin(8),
                    )
                    .collapsible(false)
                    .resizable(false)
                    .show(ctx, show);
            }
            self.form_runtime_focus.extend(rendered_focus);
        }
        self.form_last_focus = ctx.memory(|m| m.focused());
    }
}
