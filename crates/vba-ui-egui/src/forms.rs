//! Reusable form designer and real runtime controls. All edits/events are explicit IDE commands.
use super::*;
mod accelerator;
mod combo;
mod images;
pub(super) mod keyboard;
mod list;
pub(super) mod mouse;
mod numeric;
mod refedit;
mod tabs;
mod text;
use vba_ide::forms::{Align, Bounds, ControlKind, DesignEdit, SameSize};
#[derive(Clone, Copy)]
pub(super) struct KeyboardFrame {
    pub(super) owner: Option<vba_ide::FormInstance>,
    pub(super) blocked: bool,
    pub(super) owns_keys: bool,
}

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
    InstanceEvent(vba_ide::FormInstance, Option<u32>, Option<String>),
    InstanceFocus(vba_ide::FormInstance, u32),
    InstanceWindowFocus(vba_ide::FormInstance, bool),
    InstanceActivate(vba_ide::FormInstance, u32),
    InstanceNavigate(vba_ide::FormInstance, u32, i32),
    InstanceDrop(vba_ide::FormInstance, u32, bool),
    InstanceList(vba_ide::FormInstance, u32, i32),
    InstanceListSelection(vba_ide::FormInstance, u32, vba_ide::FormListInput),
    InstanceTab(vba_ide::FormInstance, u32, i32),
    InstanceNumeric(vba_ide::FormInstance, u32, vba_ide::FormNumericInput),
    InstanceMouse(vba_ide::FormInstance, Option<u32>, vba_ide::FormMouseInput),
    InstanceKey(vba_ide::FormInstance, Option<u32>, vba_ide::FormKeyInput),
    InstanceText(vba_ide::FormInstance, u32, vba_ide::VbaString, (u32, u32)),
}
impl FormCommand {
    // New form commands require host preparation unless explicitly known to be inert.
    pub(crate) fn executes_code(&self) -> bool {
        !matches!(
            self,
            Self::Add
                | Self::Edit(..)
                | Self::Select(..)
                | Self::SelectAll(..)
                | Self::Tab(..)
                | Self::Grid(..)
                | Self::Zoom(..)
                | Self::History(..)
                | Self::Handler(..)
        )
    }
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
        FormCommand::InstanceMouse(instance, control, input) => ide
            .form_instance_mouse(instance, control, input, host)
            .map(|_| ()),
        FormCommand::InstanceKey(instance, control, input) => ide
            .form_instance_key(instance, control, input, host)
            .map(|_| ()),
        FormCommand::InstanceText(instance, control, text, selection) => {
            ide.form_instance_text(instance, control, text, selection, host)
        }
        FormCommand::InstanceNavigate(instance, control, row) => {
            ide.form_instance_list_navigation(instance, control, row, host)
        }
        FormCommand::InstanceDrop(instance, control, opened) => {
            ide.form_instance_drop(instance, control, opened, host)
        }
        FormCommand::InstanceListSelection(instance, control, input) => {
            ide.form_instance_list_selection(instance, control, input, host)
        }
        FormCommand::InstanceList(instance, control, row) => {
            ide.form_instance_list(instance, control, row, host)
        }
        FormCommand::InstanceTab(instance, control, tab) => {
            ide.form_instance_tab(instance, control, tab, host)
        }
        FormCommand::InstanceNumeric(instance, control, input) => {
            ide.form_instance_numeric(instance, control, input, host)
        }
        FormCommand::InstanceActivate(instance, control) => {
            ide.form_instance_activate(instance, control, host)
        }
        FormCommand::InstanceWindowFocus(instance, active) => {
            ide.form_instance_window_focus(instance, active, host)
        }
        FormCommand::InstanceFocus(instance, control) => {
            ide.form_instance_focus(instance, control, host)
        }
        FormCommand::InstanceEvent(instance, c, text) => {
            ide.form_instance_event(instance, c, text, host)
        }
    }
}
#[derive(Clone, Debug)]
pub struct FormLayoutReport {
    pub source: SourceId,
    pub canvas: egui::Rect,
    pub controls: Vec<(u32, egui::Rect)>,
    pub handles: Vec<(u32, egui::Rect)>,
}
fn ancestors<'a>(
    form: &'a vba_ide::forms::Form,
    control: &vba_ide::forms::Control,
) -> Option<Vec<&'a vba_ide::forms::Control>> {
    let mut result = Vec::new();
    let mut parent = control.parent;
    while let Some(id) = parent {
        if result.len() >= 8 {
            return None;
        }
        let node = form.controls.iter().find(|c| {
            c.id == id
                && matches!(
                    c.kind,
                    ControlKind::Frame | ControlKind::MultiPage | ControlKind::Page
                )
        })?;
        result.push(node);
        parent = node.parent;
    }
    result.reverse();
    Some(result)
}
fn native_page_visible(
    ide: &Ide,
    instance: vba_ide::FormInstance,
    form: &vba_ide::forms::Form,
    control: &vba_ide::forms::Control,
) -> bool {
    let Some(parents) = ancestors(form, control) else {
        return false;
    };
    parents
        .into_iter()
        .chain(std::iter::once(control))
        .all(|c| {
            if ide
                .form_instance_field(instance, Some(c.id), "Visible")
                .ok()
                != Some(vba_ide::Value::Boolean(true))
            {
                return false;
            }
            if c.kind != ControlKind::Page {
                return true;
            }
            let Some(parent) = c.parent else {
                return false;
            };
            match (
                ide.form_instance_field(instance, Some(parent), "Value"),
                ide.form_instance_field(instance, Some(c.id), "VbaIndex"),
            ) {
                (Ok(vba_ide::Value::Long(selected)), Ok(vba_ide::Value::Integer(index))) => {
                    selected >= 0 && selected == i32::from(index)
                }
                _ => false,
            }
        })
}
fn native_eligible(
    ide: &Ide,
    instance: vba_ide::FormInstance,
    form: &vba_ide::forms::Form,
    control: &vba_ide::forms::Control,
    tabbing: bool,
) -> bool {
    let Some(parents) = ancestors(form, control) else {
        return false;
    };
    native_page_visible(ide, instance, form, control)
        && parents
            .into_iter()
            .chain(std::iter::once(control))
            .all(|c| {
                ["Visible", "Enabled"]
                    .into_iter()
                    .chain((tabbing && c.kind != ControlKind::Page).then_some("TabStop"))
                    .all(|name| {
                        ide.form_instance_field(instance, Some(c.id), name).ok()
                            == Some(vba_ide::Value::Boolean(true))
                    })
            })
}
fn parent_geometry(
    ide: &Ide,
    instance: vba_ide::FormInstance,
    form: &vba_ide::forms::Form,
    control: &vba_ide::forms::Control,
    rect: egui::Rect,
) -> Option<(egui::Vec2, egui::Rect)> {
    let mut offset = egui::Vec2::ZERO;
    let mut clip = egui::Rect::EVERYTHING;
    for parent in ancestors(form, control)? {
        let number = |name: &str, fallback: f32| match ide.form_instance_field(
            instance,
            Some(parent.id),
            name,
        ) {
            Ok(vba_ide::Value::Single(n)) if n.is_finite() && n.abs() <= 32767. => n,
            _ => fallback,
        };
        offset += egui::vec2(
            number("Left", parent.bounds.left),
            number("Top", parent.bounds.top),
        ) * (4. / 3.);
        let bounds = egui::Rect::from_min_size(
            rect.min + offset,
            egui::vec2(
                number("Width", parent.bounds.width).max(0.),
                number("Height", parent.bounds.height).max(0.),
            ) * (4. / 3.),
        );
        clip = clip.intersect(bounds.shrink(1.));
    }
    Some((offset, clip))
}
fn control_text(value: vba_ide::Value) -> String {
    match value {
        vba_ide::Value::String(text) => String::from_utf16_lossy(text.units()),
        value => vba_ide::display_value(&value),
    }
}
fn control_tip(
    ui: &egui::Ui,
    widget: egui::Id,
    bounds: egui::Rect,
    tip: &vba_ide::VbaString,
    theme: Theme,
) {
    if tip.units().is_empty() {
        return;
    }
    let remaining =
        ui.style().interaction.tooltip_delay - ui.input(|i| i.pointer.time_since_last_movement());
    if remaining > 0. {
        ui.ctx().request_repaint_after(
            std::time::Duration::try_from_secs_f32(remaining)
                .unwrap_or(std::time::Duration::from_millis(50)),
        );
    } else {
        let mut tooltip = egui::Tooltip::always_open(
            ui.ctx().clone(),
            ui.layer_id(),
            widget.with("control.tip"),
            bounds,
        )
        .at_pointer();
        tooltip.popup = tooltip.popup.frame(
            egui::Frame::popup(ui.style())
                .fill(theme.editor)
                .stroke(egui::Stroke::new(1., theme.border)),
        );
        tooltip.show(|ui| {
            ui.label(egui::RichText::new(String::from_utf16_lossy(tip.units())).color(theme.text));
        });
    }
}
fn command(commands: &mut Vec<Command>, id: SourceId, edit: DesignEdit) {
    commands.push(Command::Form(FormCommand::Edit(id, edit)));
}
fn string_property(
    ui: &mut egui::Ui,
    label: &str,
    value: &mut String,
    limit: usize,
    theme: Theme,
) -> bool {
    let label = ui.label(theme.label(label));
    let changed = ui
        .add(TextEdit::singleline(value).char_limit(limit))
        .labelled_by(label.id)
        .changed();
    ui.end_row();
    changed
}
fn number_property(ui: &mut egui::Ui, label: &str, value: &mut f32, theme: Theme) -> bool {
    let label = ui.label(theme.label(label));
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
        theme: Theme,
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
                if ui.button(theme.label(label)).clicked() {
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
            if ui.button(theme.label("View Form Code")).clicked() {
                self.form_view = false;
                self.focus_editor = true;
            }
            if ui
                .add_enabled(editable, egui::Button::new(theme.label("Run Form")))
                .clicked()
            {
                commands.push(Command::Form(FormCommand::Run(id, self.form_modal)));
            }
            ui.checkbox(&mut self.form_modal, theme.label("Modal"));
            if ui
                .add_enabled(
                    ide.enabled(Action::Reset),
                    egui::Button::new(theme.label("Reset Form")),
                )
                .clicked()
            {
                commands.push(Command::Action(Action::Reset));
            }
        });
        ui.add_enabled_ui(editable, |ui| {
            ui.horizontal_wrapped(|ui| {
                ui.label(theme.label("Toolbox:"));
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
                    .add_enabled(
                        designer.can_undo(),
                        egui::Button::new(theme.label("Undo Design")),
                    )
                    .clicked()
                {
                    commands.push(Command::Form(FormCommand::History(id, false)));
                }
                if ui
                    .add_enabled(
                        designer.can_redo(),
                        egui::Button::new(theme.label("Redo Design")),
                    )
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
                let changed = ui
                    .checkbox(&mut snap, theme.label("Snap to Grid"))
                    .changed();
                let label = ui.label(theme.label("Grid points"));
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
            ui.label(
                theme.label("Reset execution to edit the design. Runtime values are separate."),
            );
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
            ui.heading(theme.label("Form Properties"));
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
                                    changed |=
                                        string_property(ui, "Control Name", &mut c.name, 64, theme);
                                    if matches!(
                                        c.kind,
                                        ControlKind::Label
                                            | ControlKind::Frame
                                            | ControlKind::CommandButton
                                            | ControlKind::CheckBox
                                            | ControlKind::OptionButton
                                            | ControlKind::ToggleButton
                                    ) {
                                        changed |= string_property(
                                            ui,
                                            "Caption",
                                            &mut c.caption,
                                            1024,
                                            theme,
                                        );
                                    }
                                    if c.kind == ControlKind::TextBox {
                                        changed |=
                                            string_property(ui, "Text", &mut c.text, 16384, theme);
                                    }
                                    changed |=
                                        number_property(ui, "Left", &mut c.bounds.left, theme);
                                    changed |= number_property(ui, "Top", &mut c.bounds.top, theme);
                                    changed |=
                                        number_property(ui, "Width", &mut c.bounds.width, theme);
                                    changed |=
                                        number_property(ui, "Height", &mut c.bounds.height, theme);
                                    changed |= ui
                                        .checkbox(&mut c.enabled, theme.label("Enabled"))
                                        .changed();
                                    ui.end_row();
                                    changed |= ui
                                        .checkbox(&mut c.visible, theme.label("Visible"))
                                        .changed();
                                    ui.end_row();
                                    changed |= ui
                                        .checkbox(&mut c.tab_stop, theme.label("Tab Stop"))
                                        .changed();
                                    ui.end_row();
                                });
                            if changed {
                                command(commands, id, DesignEdit::Replace(c));
                            }
                            if ui.button(theme.label("View Default Event")).clicked() {
                                commands.push(Command::Form(FormCommand::Handler(
                                    id,
                                    Some(control.id),
                                    control.kind.default_event().into(),
                                )));
                                self.form_view = false;
                                self.navigate = true;
                            }
                            if ui.button(theme.label("Delete Selection")).clicked() {
                                command(commands, id, DesignEdit::Delete);
                            }
                        } else {
                            let mut caption = form.caption.clone();
                            let mut width = form.width;
                            let mut height = form.height;
                            let mut changed = false;
                            ui.label(&form.name);
                            egui::Grid::new("form.props").num_columns(2).show(ui, |ui| {
                                changed |=
                                    string_property(ui, "Form Caption", &mut caption, 1024, theme);
                                changed |= number_property(ui, "Form Width", &mut width, theme);
                                changed |= number_property(ui, "Form Height", &mut height, theme);
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
                        ui.heading(theme.label("Tab Order"));
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
                    let Some(parents) = ancestors(form, c) else {
                        continue;
                    };
                    let mut offset = egui::Vec2::ZERO;
                    let previous_clip = ui.clip_rect();
                    let mut clip = previous_clip;
                    for parent in parents {
                        offset += egui::vec2(parent.bounds.left, parent.bounds.top) * scale;
                        clip = clip.intersect(
                            egui::Rect::from_min_size(
                                rect.min + offset,
                                egui::vec2(parent.bounds.width, parent.bounds.height) * scale,
                            )
                            .shrink(1.),
                        );
                    }
                    ui.set_clip_rect(clip);
                    let bounds = egui::Rect::from_min_size(
                        rect.min + offset + egui::vec2(c.bounds.left, c.bounds.top) * scale,
                        egui::vec2(c.bounds.width, c.bounds.height) * scale,
                    );
                    let label = format!("{} ({})", c.name, c.kind.name());
                    let selected = designer.selection().contains(&c.id);
                    let response = if c.kind == ControlKind::Frame {
                        let response = ui.interact(
                            bounds,
                            ui.scope_id().with(("form.frame", c.id)),
                            egui::Sense::click_and_drag(),
                        );
                        ui.painter().rect_filled(bounds, 0., theme.chrome);
                        ui.painter().rect_stroke(
                            bounds.shrink(1.),
                            0.,
                            egui::Stroke::new(1., theme.border),
                            egui::StrokeKind::Inside,
                        );
                        ui.painter().text(
                            bounds.left_top() + egui::vec2(6., 3.),
                            egui::Align2::LEFT_TOP,
                            &c.caption,
                            egui::TextStyle::Body.resolve(ui.style()),
                            theme.text,
                        );
                        response
                    } else {
                        ui.put(
                            bounds,
                            egui::Button::new(if c.kind == ControlKind::TextBox {
                                format!("{}: {}", c.name, c.text)
                            } else {
                                format!("{}: {}", c.name, c.caption)
                            })
                            .selected(selected)
                            .sense(egui::Sense::click_and_drag()),
                        )
                    }
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
                    ui.set_clip_rect(previous_clip);
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
    /// Build an executable focus command after the embedding activates its document window.
    /// The embedding must dispatch with normal authorization before handling document input.
    /// OS/viewport activation is deliberately not an application window transition.
    pub fn document_window_activated(&self, ide: &Ide) -> Option<Command> {
        let owner = ide.form_active_window()?;
        (ide.form_instance_can_deliver(owner) && !ide.form_instance_has_modal_window(owner))
            .then_some(Command::Form(FormCommand::InstanceWindowFocus(
                owner, false,
            )))
    }
    pub(super) fn form_keyboard_input(
        &mut self,
        ctx: &egui::Context,
        ide: &Ide,
        commands: &mut Vec<Command>,
    ) -> KeyboardFrame {
        if matches!(ide.mode(), Mode::Completed | Mode::Waiting) {
            let active = ide.form_active_window();
            if self.form_last_active_window != active {
                self.form_last_active_window = active;
                if let Some(owner) = active {
                    let control = ide
                        .form_instance_field(owner, None, "VbaActiveControl")
                        .ok();
                    let widget = self
                        .form_runtime_focus
                        .iter()
                        .find(|(form, id, _)| {
                            *form == owner
                                && ide
                                    .form_instance_control_object(owner, *id)
                                    .ok()
                                    .map(vba_ide::Value::Object)
                                    == control
                        })
                        .map(|(_, _, widget)| *widget)
                        .or_else(|| {
                            self.form_runtime_roots
                                .iter()
                                .find(|(form, _)| *form == owner)
                                .map(|(_, widget)| *widget)
                        });
                    if let Some(widget) = widget {
                        ctx.memory_mut(|memory| memory.request_focus(widget));
                    }
                }
            }
        }
        self.form_mouse
            .step(ctx, ide, commands, self.form_keys.pending_owner(ide));
        // A completed pointer callback may redirect focus. Restore the canonical
        // decision before egui can turn its pre-callback focus into another input.
        if let Some(owner) = self
            .form_mouse
            .focus_owner
            .filter(|owner| ide.form_instance_can_deliver(*owner))
        {
            let active = ide
                .form_instance_field(owner, None, "VbaActiveControl")
                .ok();
            if let Some((_, _, widget)) =
                self.form_runtime_focus.iter().find(|(form, control, _)| {
                    *form == owner
                        && ide
                            .form_instance_control_object(owner, *control)
                            .ok()
                            .map(vba_ide::Value::Object)
                            == active
                })
            {
                ctx.memory_mut(|m| m.request_focus(*widget));
                self.form_mouse.keyboard_focus = Some(*widget);
            }
        }
        if ctx.memory(|memory| memory.focused()).is_none()
            && let Some(widget) = self.form_mouse.keyboard_focus.filter(|widget| {
                self.form_runtime_focus
                    .iter()
                    .any(|(_, _, id)| id == widget)
                    || self.form_runtime_roots.iter().any(|(_, id)| id == widget)
            })
        {
            ctx.memory_mut(|memory| memory.request_focus(widget));
        }
        if self.form_mouse.idle() {
            self.form_mouse.keyboard_focus = None;
            self.form_mouse.focus_owner = None;
        }
        if matches!(ide.mode(), Mode::Design | Mode::Failed | Mode::Stale) {
            self.form_keys.clear();
            self.form_last_active_window = None;
            self.form_runtime_roots.clear();
            return KeyboardFrame {
                owner: None,
                blocked: false,
                owns_keys: false,
            };
        }
        // egui clears focus for Escape at the start of the pass. Recover only a
        // previously focused native widget, so its VBA event can decide the action.
        if ctx.memory(|memory| memory.focused()).is_none()
            && ctx.input(|input| {
                input.events.iter().any(|event| {
                    matches!(
                        event,
                        egui::Event::Key {
                            key: egui::Key::Escape,
                            pressed: true,
                            ..
                        }
                    )
                })
            })
            && let Some(previous) = self.form_last_focus.filter(|id| {
                self.form_runtime_focus
                    .iter()
                    .any(|(_, _, widget)| widget == id)
                    || self
                        .form_runtime_roots
                        .iter()
                        .any(|(_, widget)| widget == id)
            })
        {
            ctx.memory_mut(|memory| memory.request_focus(previous));
        }
        let focused = ctx.memory(|memory| memory.focused());
        let instances = ide.form_instances().unwrap_or_default();
        let owner = instances
            .iter()
            .copied()
            .find(|instance| ide.form_instance_is_modal(*instance))
            .or_else(|| {
                instances
                    .iter()
                    .filter_map(|instance| {
                        ide.form_instance_modal_rank(*instance)
                            .map(|rank| (rank, *instance))
                    })
                    .max_by_key(|(rank, _)| *rank)
                    .map(|(_, instance)| instance)
            })
            .or_else(|| self.form_keys.pending_owner(ide))
            .or_else(|| ide.form_active_window())
            .or_else(|| {
                self.form_runtime_focus
                    .iter()
                    .find(|(_, _, id)| Some(*id) == focused)
                    .map(|(owner, _, _)| *owner)
            })
            .or_else(|| {
                self.form_runtime_roots
                    .iter()
                    .find(|(_, id)| Some(*id) == focused)
                    .map(|(owner, _)| *owner)
            })
            .or_else(|| {
                instances
                    .iter()
                    .rev()
                    .copied()
                    .find(|instance| ide.form_instance_visible(*instance))
            });
        let blocked = self.form_keys.step(
            ctx,
            ide,
            owner,
            keyboard::Targets {
                focus: &self.form_runtime_focus,
                roots: &self.form_runtime_roots,
            },
            commands,
            !owner.is_some_and(|owner| self.form_mouse.blocks(owner)),
        );
        let focused = ctx.memory(|memory| memory.focused());
        let owns_keys = ide.mode() != Mode::Break
            && owner.is_some_and(|owner| {
                ide.native_form(owner.source).is_some()
                    && (blocked
                        || ide.form_instance_is_modal(owner)
                        || self
                            .form_runtime_focus
                            .iter()
                            .any(|(instance, _, id)| *instance == owner && Some(*id) == focused)
                        || self
                            .form_runtime_roots
                            .iter()
                            .any(|(instance, id)| *instance == owner && Some(*id) == focused))
            });
        KeyboardFrame {
            owner,
            blocked,
            owns_keys,
        }
    }
    pub(super) fn form_windows(
        &mut self,
        ctx: &egui::Context,
        ide: &Ide,
        commands: &mut Vec<Command>,
        theme: Theme,
        input: Option<KeyboardFrame>,
    ) {
        let input = input.unwrap_or_else(|| self.form_keyboard_input(ctx, ide, commands));
        if matches!(ide.mode(), Mode::Break | Mode::Stale) {
            if ide.mode() == Mode::Stale {
                self.form_runtime_focus.clear();
                self.form_runtime_roots.clear();
                self.form_active_control.clear();
                self.form_last_focus = None;
            }
            return;
        }
        self.form_mouse.hits.clear();
        let previous_active = std::mem::take(&mut self.form_active_control);
        let previous_focus = std::mem::take(&mut self.form_runtime_focus);
        let previous_roots = std::mem::take(&mut self.form_runtime_roots);
        let last_focus = ctx
            .memory(|memory| memory.focused())
            .or(self.form_last_focus.take());
        let Ok(mut instances) = ide.form_instances() else {
            return;
        };
        instances.sort_by_key(|instance| {
            ide.form_instance_modal_rank(*instance).map_or_else(
                || {
                    let order = ide
                        .form_instance_field(*instance, None, "VbaWindowOrder")
                        .ok()
                        .and_then(|v| {
                            if let vba_ide::Value::Long(n) = v {
                                usize::try_from(n).ok()
                            } else {
                                None
                            }
                        })
                        .unwrap_or(0);
                    (false, order)
                },
                |rank| (true, rank),
            )
        });
        let keyboard_owner = input.owner;
        let key_blocked = input.blocked;
        for instance in instances {
            let id = instance.source;
            let Some(designer) = ide.form(id).filter(|_| ide.form_instance_visible(instance))
            else {
                continue;
            };
            let Ok(runtime_form) = ide.form_instance_design(instance) else {
                continue;
            };
            let form = &runtime_form;
            let _ = designer;
            let idle = ide.form_instance_can_deliver(instance)
                && (!self.form_mouse.blocks(instance)
                    || self.form_keys.busy() && !self.form_mouse.pending())
                && !(key_blocked && keyboard_owner == Some(instance));
            let native = ide.native_form(id).is_some();
            if let Some((owner, control)) = ide
                .form_range_selection()
                .ok()
                .flatten()
                .filter(|(owner, _)| *owner == instance)
            {
                if let Some(c) = form.controls.iter().find(|c| c.id == control) {
                    egui::Window::new(
                        ide.form_instance_field(instance, None, "Caption")
                            .map(control_text)
                            .unwrap_or_else(|_| form.caption.clone()),
                    )
                    .id(self.scope.unwrap_or(self.instance).with((
                        "refedit",
                        owner.object.object,
                        owner.object.generation,
                    )))
                    .collapsible(false)
                    .resizable(false)
                    .title_bar(false)
                    .frame(
                        egui::Frame::new()
                            .fill(theme.chrome)
                            .stroke(egui::Stroke::new(1., theme.border))
                            .corner_radius(5)
                            .inner_margin(8),
                    )
                    .show(ctx, |ui| {
                        window_theme(ui, theme);
                        ui.heading(
                            ide.form_instance_field(instance, None, "Caption")
                                .map(control_text)
                                .unwrap_or_else(|_| form.caption.clone()),
                        );
                        ui.add_enabled_ui(idle && self.range_selection_enabled, |ui| {
                            let (bounds, _) =
                                ui.allocate_exact_size(egui::vec2(320., 26.), egui::Sense::hover());
                            let widget = ui.scope_id().with(c.id);
                            self.form_runtime_focus.push((instance, c.id, widget));
                            if idle
                                && !previous_focus.iter().any(|(owner, _, target)| {
                                    *owner == instance && *target == widget
                                })
                            {
                                ui.ctx().memory_mut(|m| m.request_focus(widget));
                            }
                            if let Ok(object) = ide.form_instance_control_object(instance, c.id) {
                                self.form_mouse.hit(mouse::Hit {
                                    owner: instance,
                                    control: Some(c.id),
                                    object,
                                    rect: bounds.intersect(ui.clip_rect()),
                                    origin: bounds.min,
                                    parent: bounds.min,
                                    kind: Some(c.kind),
                                    part: mouse::Part::Body,
                                });
                            }
                            let regions = refedit::RefEdit {
                                text: text::TextBox {
                                    ide,
                                    instance,
                                    control: c,
                                    bounds,
                                    widget,
                                    enabled: true,
                                    theme,
                                },
                                picker: true,
                            }
                            .show(ui, commands);
                            self.form_mouse.regions(instance, c.id, regions);
                            if idle
                                && ui.input_mut(|i| {
                                    i.consume_key(egui::Modifiers::NONE, egui::Key::Escape)
                                        || i.consume_key(egui::Modifiers::NONE, egui::Key::Enter)
                                })
                            {
                                commands.push(Command::Form(FormCommand::InstanceDrop(
                                    instance, c.id, false,
                                )));
                            }
                        });
                    });
                }
                continue;
            }

            if native {
                let active = match ide.form_instance_field(instance, None, "VbaActiveControl") {
                    Ok(vba_ide::Value::Object(object)) => Some(object),
                    _ => None,
                };
                let changed = previous_active
                    .iter()
                    .find(|(owner, _)| *owner == instance)
                    .is_none_or(|(_, old)| *old != active);
                self.form_active_control.push((instance, active));
                if idle
                    && changed
                    && keyboard_owner == Some(instance)
                    && let Some((_, _, target)) =
                        previous_focus.iter().find(|(owner, control, _)| {
                            *owner == instance
                                && form.controls.iter().find(|c| c.id == *control).is_some_and(
                                    |c| {
                                        ide.form_instance_control_object(instance, c.id)
                                            .ok()
                                            .map(vba_ide::Value::Object)
                                            == active.map(vba_ide::Value::Object)
                                    },
                                )
                        })
                {
                    ctx.memory_mut(|m| m.request_focus(*target));
                }
            }
            let editing = if native && idle {
                previous_focus
                    .iter()
                    .find(|(owner, _, target)| *owner == instance && Some(*target) == last_focus)
                    .and_then(|(_, id, _)| {
                        form.controls.iter().find(|c| {
                            c.id == *id
                                && matches!(c.kind, ControlKind::TextBox | ControlKind::ComboBox)
                                && native_eligible(ide, instance, form, c, false)
                        })
                    })
            } else {
                None
            };
            let text_keys = editing
                .map(|c| {
                    if c.kind == ControlKind::ComboBox {
                        combo::keyboard(ctx, ide, instance, c.id, commands)
                    } else {
                        text::keyboard(ctx, ide, instance, c.id)
                    }
                })
                .unwrap_or_default();
            let mut ordered = previous_focus
                .iter()
                .filter(|(source, control, _)| {
                    *source == instance
                        && (!native
                            || form
                                .controls
                                .iter()
                                .find(|c| c.id == *control)
                                .is_some_and(|c| native_eligible(ide, instance, form, c, true)))
                        && (!native
                            || ide
                                .form_instance_field(instance, Some(*control), "TabStop")
                                .ok()
                                == Some(vba_ide::Value::Boolean(true)))
                })
                .map(|(_, control, target)| {
                    let index = match ide.form_instance_field(
                        instance,
                        Some(*control),
                        if native {
                            "VbaNavigationRank"
                        } else {
                            "TabIndex"
                        },
                    ) {
                        Ok(vba_ide::Value::Integer(n)) => i32::from(n),
                        Ok(vba_ide::Value::Long(n)) => n,
                        _ => i32::MAX,
                    };
                    (index, *control, *target)
                })
                .collect::<Vec<_>>();
            ordered.sort_by_key(|(index, control, _)| (*index, *control));
            if idle
                && (!native
                    || previous_focus.iter().any(|(owner, _, target)| {
                        *owner == instance && Some(*target) == last_focus
                    }))
                && let Some(at) = ordered.iter().position(|(_, control, target)| {
                    if native {
                        form.controls
                            .iter()
                            .find(|c| c.id == *control)
                            .is_some_and(|c| {
                                ide.form_instance_control_object(instance, c.id)
                                    .ok()
                                    .map(vba_ide::Value::Object)
                                    == ide
                                        .form_instance_field(instance, None, "VbaActiveControl")
                                        .ok()
                            })
                    } else {
                        Some(*target) == last_focus
                    }
                })
            {
                for (modifiers, backward) in [
                    (egui::Modifiers::SHIFT, true),
                    (egui::Modifiers::NONE, false),
                ] {
                    if !text_keys.1 && ctx.input_mut(|i| i.consume_key(modifiers, egui::Key::Tab)) {
                        let next = if backward {
                            (at + ordered.len() - 1) % ordered.len()
                        } else {
                            (at + 1) % ordered.len()
                        };
                        if let Some((_, control, target)) = ordered.get(next) {
                            if native {
                                commands.push(Command::Form(FormCommand::InstanceFocus(
                                    instance, *control,
                                )));
                            } else {
                                ctx.memory_mut(|m| {
                                    m.move_focus(egui::FocusDirection::None);
                                    m.request_focus(*target);
                                });
                            }
                        }
                    }
                }
            }
            if editing.is_some()
                && !text_keys.0
                && !ordered.is_empty()
                && !form.controls.iter().any(|c| {
                    c.kind == ControlKind::CommandButton
                        && ide
                            .form_instance_field(instance, Some(c.id), "Default")
                            .ok()
                            == Some(vba_ide::Value::Boolean(true))
                })
                && ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::Enter))
            {
                let at = ordered
                    .iter()
                    .position(|(_, _, target)| Some(*target) == last_focus)
                    .unwrap_or(0);
                if let Some((_, control, _)) = ordered.get((at + 1) % ordered.len()) {
                    commands.push(Command::Form(FormCommand::InstanceFocus(
                        instance, *control,
                    )));
                }
            }
            if idle && keyboard_owner == Some(instance) {
                for (key, property) in
                    [(egui::Key::Enter, "Default"), (egui::Key::Escape, "Cancel")]
                {
                    if key == egui::Key::Enter && text_keys.0 {
                        continue;
                    }
                    let focused = if key == egui::Key::Enter {
                        previous_focus
                            .iter()
                            .find(|(owner, _, target)| {
                                *owner == instance && Some(*target) == last_focus
                            })
                            .and_then(|(_, control, _)| {
                                form.controls.iter().find(|c| {
                                    c.id == *control && c.kind == ControlKind::CommandButton
                                })
                            })
                    } else {
                        None
                    };
                    let chosen = focused.or_else(|| {
                        form.controls.iter().find(|c| {
                            c.kind == ControlKind::CommandButton
                                && ide.form_instance_field(instance, Some(c.id), property).ok()
                                    == Some(vba_ide::Value::Boolean(true))
                        })
                    });
                    if let Some(control) = chosen.filter(|c| {
                        if native && !native_eligible(ide, instance, form, c, false) {
                            return false;
                        }
                        ["Enabled", "Visible"].iter().all(|property| {
                            ide.form_instance_field(instance, Some(c.id), property).ok()
                                == Some(vba_ide::Value::Boolean(true))
                        })
                    }) && ctx.input_mut(|input| input.consume_key(egui::Modifiers::NONE, key))
                    {
                        commands.push(Command::Form(if native {
                            FormCommand::InstanceActivate(instance, control.id)
                        } else {
                            FormCommand::InstanceEvent(instance, Some(control.id), None)
                        }));
                    }
                }
            }
            let mut rendered_focus = Vec::new();
            let mut rendered_root = None;
            let show = |ui: &mut egui::Ui| {
                window_theme(ui, theme);
                ui.horizontal(|ui| {
                    ui.heading(
                        ide.form_instance_field(instance, None, "Caption")
                            .map(control_text)
                            .unwrap_or_else(|_| form.caption.clone()),
                    );
                    if ui
                        .add_enabled(idle, egui::Button::new(theme.label("Close Form")))
                        .clicked()
                    {
                        commands.push(Command::Form(FormCommand::InstanceEvent(
                            instance, None, None,
                        )));
                    }
                });
                ui.add_enabled_ui(idle, |ui| {
                    let scale = 4.0 / 3.0;
                    let dimension = |name: &str, fallback: f32| match ide
                        .form_instance_field(instance, None, name)
                    {
                        Ok(vba_ide::Value::Single(n)) => n,
                        _ => fallback,
                    };
                    let (rect, _) = ui.allocate_exact_size(
                        egui::vec2(
                            dimension("Width", form.width) * scale,
                            dimension("Height", form.height) * scale,
                        ),
                        egui::Sense::hover(),
                    );
                    if native {
                        self.form_mouse.hit(mouse::Hit{owner:instance,control:None,object:instance.object,rect:rect.intersect(ui.clip_rect()),origin:rect.min,parent:rect.min,kind:None,part:mouse::Part::Body});
                    }
                    let mut focus = Vec::new();
                    let root_eligible = native
                        && ide.form_keyboard_target(instance).ok() == Some(None)
                        && !form.controls.iter().any(|control| native_eligible(ide,instance,form,control,false));
                    if root_eligible {
                        let response=ui.interact(rect,ui.scope_id().with("runtime.form.keyboard"),egui::Sense::click());
                        response.widget_info(|| egui::WidgetInfo::labeled(egui::accesskit::Role::Pane,idle,&form.name));
                        if idle && (response.clicked() || keyboard_owner==Some(instance) && !previous_roots.iter().any(|(owner,_)| *owner==instance)) {
                            response.request_focus();
                        }
                        rendered_root=Some(response.id);
                    }
                    for c in &form.controls {
                        if native && !native_page_visible(ide,instance,form,c) {continue;}
                        let Some((offset,parent_clip))=parent_geometry(ide,instance,form,c,rect) else {continue;};
                        if ide
                            .form_instance_field(instance, Some(c.id), "Visible")
                            .ok()
                            != Some(vba_ide::Value::Boolean(true))
                        {
                            continue;
                        }
                        let enabled = (!native || native_eligible(ide,instance,form,c,false)) && ide
                            .form_instance_field(instance, Some(c.id), "Enabled")
                            .ok()
                            == Some(vba_ide::Value::Boolean(true));
                        let caption = ide
                            .form_instance_field(instance, Some(c.id), "Caption")
                            .map(control_text)
                            .unwrap_or_else(|_| c.caption.clone());
                        let number = |name: &str, fallback: f32| match ide.form_instance_field(
                            instance,
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
                            rect.min+offset
                                + egui::vec2(
                                    number("Left", c.bounds.left),
                                    number("Top", c.bounds.top),
                                ) * scale,
                            egui::vec2(
                                number("Width", c.bounds.width).max(0.0),
                                number("Height", c.bounds.height).max(0.0),
                            ) * scale,
                        );
                        if native && enabled && let Ok(object)=ide.form_instance_control_object(instance,c.id) {
                            self.form_mouse.hit(mouse::Hit{owner:instance,control:Some(c.id),object,rect:bounds.intersect(parent_clip).intersect(ui.clip_rect()),origin:bounds.min,parent:rect.min+offset,kind:Some(c.kind),part:mouse::Part::Body});
                        }
                        let widget_id = ui.scope_id().with(("runtime.control", c.id));
                        if native && idle && keyboard_owner==Some(instance)
                            && !previous_focus.iter().any(|(owner,_,_)|*owner==instance)
                            && matches!(c.kind,ControlKind::TextBox|ControlKind::SpinButton|ControlKind::ScrollBar|ControlKind::ListBox|ControlKind::ComboBox)
                            && ide.form_instance_control_object(instance,c.id).ok().map(vba_ide::Value::Object)==ide.form_instance_field(instance,None,"VbaActiveControl").ok() {
                            ui.ctx().memory_mut(|m|m.request_focus(widget_id));
                        }
                        let tab_stop = ide
                            .form_instance_field(instance, Some(c.id), "TabStop")
                            .ok()
                            == Some(vba_ide::Value::Boolean(true));
                        let index = match ide.form_instance_field(instance, Some(c.id), if native {"VbaNavigationRank"} else {"TabIndex"})
                        {
                            Ok(vba_ide::Value::Integer(n)) => i32::from(n),
                            Ok(vba_ide::Value::Long(n)) => n,
                            _ => i32::MAX,
                        };
                        if enabled && if native { c.kind != ControlKind::Label && !matches!(c.kind,ControlKind::Frame|ControlKind::Page|ControlKind::Image) } else { tab_stop } {
                            focus.push((index, c.id, widget_id));
                        }
                        let previous_clip=ui.clip_rect();
                        ui.set_clip_rect(previous_clip.intersect(parent_clip));
                        let control_scope=egui::UiBuilder::new().id_salt(("runtime",c.id));
                        let control_scope=if let Some(parent)=c.parent {control_scope.accessibility_parent(ui.scope_id().with(("runtime.control",parent)))} else {control_scope};
                        {
                            let mut control_ui=ui.new_child(control_scope.max_rect(bounds));
                            control_ui.add_enabled_ui(enabled, |ui| match c.kind {
                                ControlKind::Frame | ControlKind::Page=> {
                                    let response=ui.interact(bounds,widget_id,if c.kind==ControlKind::Page {egui::Sense::hover()}else{egui::Sense::click()});
                                    ui.painter().rect_filled(bounds,2.,theme.chrome);
                                    ui.painter().rect_stroke(bounds.shrink(1.),2.,egui::Stroke::new(1.,theme.border),egui::StrokeKind::Inside);
                                    if c.kind==ControlKind::Frame {ui.painter().text(bounds.left_top()+egui::vec2(6.,3.),egui::Align2::LEFT_TOP,&caption,egui::TextStyle::Body.resolve(ui.style()),theme.text);}
                                    response.widget_info(||egui::WidgetInfo::labeled(egui::accesskit::Role::Group,enabled,&c.name));
                                    if response.clicked() {commands.push(Command::Form(FormCommand::InstanceEvent(instance,Some(c.id),None)));}
                                }
                                ControlKind::TabStrip | ControlKind::MultiPage=> {let regions=tabs::Tabs{ide,instance,control:c,bounds,widget:widget_id,theme}.show(ui,commands);self.form_mouse.regions(instance,c.id,regions);}

                                ControlKind::RefEdit => {let regions=refedit::RefEdit{text:text::TextBox{ide,instance,control:c,bounds,widget:widget_id,enabled,theme},picker:self.range_selection_enabled && ide.form_instance_has_modal_window(instance)}.show(ui,commands);self.form_mouse.regions(instance,c.id,regions);}
                                ControlKind::Image=> {images::Image{ide,instance,control:c,bounds,widget:widget_id,theme}.show(ui,&mut self.form_picture_textures,commands);}
                                ControlKind::ComboBox=> {let regions=combo::ComboBox{ide,instance,control:c,bounds,widget:widget_id,theme}.show(ui,commands);self.form_mouse.regions(instance,c.id,regions);}
                                ControlKind::ListBox => {let regions=list::ListBox{ide,instance,control:c,bounds,widget:widget_id,theme}.show(ui,commands);self.form_mouse.regions(instance,c.id,regions);}
                                ControlKind::SpinButton|ControlKind::ScrollBar=> {let regions=numeric::Numeric{ide,instance,control:c,bounds,widget:widget_id,theme}.show(ui,commands);self.form_mouse.regions(instance,c.id,regions);}
                                ControlKind::TextBox if native => { if let Some(geometry)= (text::TextBox {ide,instance,control:c,bounds,widget:widget_id,enabled,theme}).show(ui,commands) {self.form_mouse.text_geometry(instance,c.id,geometry);} }
                                ControlKind::TextBox => {
                                    let mut text = ide
                                        .form_instance_field(instance, Some(c.id), "VbaText")
                                        .map(control_text)
                                        .unwrap_or_default();
                                    let locked = ide
                                        .form_instance_field(instance, Some(c.id), "Locked")
                                        .ok()
                                        == Some(vba_ide::Value::Boolean(true));
                                    let max_length = match ide.form_instance_field(
                                        instance,
                                        Some(c.id),
                                        "MaxLength",
                                    ) {
                                        Ok(vba_ide::Value::Long(n)) if n > 0 => {
                                            usize::try_from(n).unwrap_or(16384).min(16384)
                                        }
                                        _ => 16384,
                                    };
                                    let readonly_copy = text.clone();
                                    let mut readonly = readonly_copy.as_str();
                                    let buffer: &mut dyn egui::TextBuffer =
                                        if locked { &mut readonly } else { &mut text };
                                    let response = ui.put(
                                        bounds,
                                        TextEdit::singleline(buffer)
                                            .id(widget_id)
                                            .char_limit(max_length)
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
                                        commands.push(Command::Form(FormCommand::InstanceEvent(
                                            instance,
                                            Some(c.id),
                                            Some(text),
                                        )));
                                    }
                                }
                                ControlKind::CheckBox
                                | ControlKind::OptionButton
                                | ControlKind::ToggleButton => {
                                    let value = ide
                                        .form_instance_field(instance, Some(c.id), "Value")
                                        .unwrap_or(vba_ide::Value::Boolean(false));
                                    let mut selected = value == vba_ide::Value::Boolean(true);
                                    let response = ui
                                        .scope(|ui| {
                                            // Functional outlines use the injected text token.
                                            let widgets = &mut ui.visuals_mut().widgets;
                                            for visual in [
                                                &mut widgets.inactive,
                                                &mut widgets.hovered,
                                                &mut widgets.active,
                                                &mut widgets.noninteractive,
                                            ] {
                                                visual.bg_stroke =
                                                    egui::Stroke::new(1., theme.text);
                                            }
                                            match c.kind {
                                                ControlKind::CheckBox => ui.put(
                                                    bounds,
                                                    egui::Checkbox::new(&mut selected, accelerator::job(&caption,ide.form_instance_field(instance,Some(c.id),"Accelerator").ok(),egui::TextStyle::Body.resolve(ui.style()),theme.text))
                                                        .indeterminate(
                                                            value == vba_ide::Value::Null,
                                                        ),
                                                ),
                                                ControlKind::OptionButton => ui.put(
                                                    bounds,
                                                    egui::RadioButton::new(selected, accelerator::job(&caption,ide.form_instance_field(instance,Some(c.id),"Accelerator").ok(),egui::TextStyle::Body.resolve(ui.style()),theme.text)),
                                                ),
                                                _ => ui.put(
                                                    bounds,
                                                    egui::Button::new(accelerator::job(&caption,ide.form_instance_field(instance,Some(c.id),"Accelerator").ok(),egui::TextStyle::Button.resolve(ui.style()),theme.text)).selected(selected),
                                                ),
                                            }
                                        })
                                        .inner;
                                    if let Some(target) =
                                        focus.iter_mut().find(|(_, control, _)| *control == c.id)
                                    {
                                        target.2 = response.id;
                                    }
                                    if response.clicked() {
                                        commands.push(Command::Form(FormCommand::InstanceEvent(
                                            instance,
                                            Some(c.id),
                                            None,
                                        )));
                                    }
                                }
                                ControlKind::Label | ControlKind::CommandButton => {
                                    let response = ui.put(
                                        bounds,
                                        egui::Button::new(accelerator::job(&caption,ide.form_instance_field(instance,Some(c.id),"Accelerator").ok(),egui::TextStyle::Button.resolve(ui.style()),theme.text))
                                            .frame(c.kind == ControlKind::CommandButton),
                                    );
                                    if let Some(target) =
                                        focus.iter_mut().find(|(_, control, _)| *control == c.id)
                                    {
                                        target.2 = response.id;
                                    }
                                    if (native && c.kind == ControlKind::Label) || (!native && !tab_stop) {
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
                                        commands.push(Command::Form(FormCommand::InstanceEvent(
                                            instance,
                                            Some(c.id),
                                            None,
                                        )));
                                    }
                                }
                            });
                        }
                        if native && !matches!(c.kind,ControlKind::TabStrip|ControlKind::MultiPage) && enabled && ui.rect_contains_pointer(bounds)
                            && !ui.input(|i| i.pointer.any_down())
                            && let Ok(vba_ide::Value::String(tip)) = ide.form_instance_field(instance, Some(c.id), "ControlTipText")
                            && !tip.units().is_empty() {
                            control_tip(ui,widget_id,bounds,&tip,theme);
                        }
                        ui.set_clip_rect(previous_clip);
                    }
                    if native && idle && keyboard_owner == Some(instance) {
                        let focused = ui.ctx().memory(|m| m.focused());
                        let active = ide.form_instance_field(instance, None, "VbaActiveControl").ok();
                        let active_id = focus.iter().find(|(_, control, _)| form.controls.iter().find(|c| c.id == *control).is_some_and(|c| ide.form_instance_control_object(instance,c.id).ok().map(vba_ide::Value::Object) == active));
                        let requested = focus.iter().find(|(_, _, target)| Some(*target) == focused);
                        let pending = commands.iter().any(|command| matches!(command, Command::Form(FormCommand::InstanceWindowFocus(owner, _) | FormCommand::InstanceMouse(owner, _, _) | FormCommand::InstanceKey(owner, _, _) | FormCommand::InstanceFocus(owner, _) | FormCommand::InstanceActivate(owner, _) | FormCommand::InstanceText(owner, _, _, _) | FormCommand::InstanceNumeric(owner, _, _) | FormCommand::InstanceList(owner, _, _) | FormCommand::InstanceListSelection(owner, _, _) | FormCommand::InstanceTab(owner, _, _) | FormCommand::InstanceDrop(owner, _, _) | FormCommand::InstanceNavigate(owner, _, _) | FormCommand::InstanceEvent(owner, _, _)) if *owner == instance));
                        if let Some((_, control, _)) = requested && requested != active_id && !pending {
                            commands.push(Command::Form(FormCommand::InstanceFocus(instance, *control)));
                        }
                        // VBA owns the focus decision, including cancellation and debugger waits.
                        if let Some((_, _, target)) = active_id
                            && focused != Some(*target)
                            && (requested.is_some() || focused.is_none() || pending || !previous_focus.iter().any(|(owner,_,_)| *owner==instance)) {
                            ui.ctx().memory_mut(|m| m.request_focus(*target));
                        }
                    }
                    rendered_focus.extend(
                        focus
                            .into_iter()
                            .map(|(_, control, target)| (instance, control, target)),
                    );
                });
                if ui.button(theme.label("Reset Form Execution")).clicked() {
                    commands.push(Command::Action(Action::Reset));
                }
            };
            let scope = self.scope.unwrap_or(self.instance).with((
                "form.runtime",
                id.0,
                instance.object.object,
                instance.object.generation,
            ));
            if ide.form_instance_has_modal_window(instance) {
                // Match retained Show order even if a held parent pointer raised its area.
                ctx.move_to_top(egui::LayerId::new(egui::Order::Foreground, scope));
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
                if native {
                    ctx.move_to_top(egui::LayerId::new(egui::Order::Middle, scope));
                }
                let window = egui::Window::new(
                    ide.form_instance_field(instance, None, "Caption")
                        .map(control_text)
                        .unwrap_or_else(|_| form.caption.clone()),
                )
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
                if native
                    && idle
                    && !ide.form_instance_window_active(instance)
                    && let Some(window) = window
                    && ctx.input(|input| input.pointer.any_pressed())
                    && let Some(pos) = ctx.input(|input| input.pointer.interact_pos())
                    && window.response.rect.contains(pos)
                    && ctx.layer_id_at(pos) == Some(window.response.layer_id)
                    && !commands.iter().any(|command| command.executes_code())
                {
                    commands.push(Command::Form(FormCommand::InstanceWindowFocus(
                        instance, true,
                    )));
                }
            }
            self.form_runtime_focus.extend(rendered_focus);
            if let Some(root) = rendered_root {
                self.form_runtime_roots.push((instance, root));
            }
        }
        self.form_last_focus = ctx.memory(|m| m.focused());
    }
}
