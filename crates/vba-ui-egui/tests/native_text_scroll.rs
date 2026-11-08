//! Real native viewport widgets, pointer callbacks and canonical text/selection.
#[path = "../../vba-project/tests/support/native_forms.rs"]
mod fixture;
use egui_kittest::{Harness, kittest::Queryable};
use vba_ide::{Action, Ide, Mode, NativeFormResource, NoHost, Value};
use vba_ui_egui::{EditorUi, Theme, apply};
type R<T = ()> = Result<T, Box<dyn std::error::Error>>;
struct State {
    ide: Ide,
    view: EditorUi,
    errors: Vec<String>,
}
fn app(
    mode: i32,
    multi: bool,
    wrap: bool,
    short: bool,
    stopped: bool,
) -> R<Harness<'static, State>> {
    app_policy(mode, multi, wrap, short, stopped, false)
}
fn app_policy(
    mode: i32,
    multi: bool,
    wrap: bool,
    short: bool,
    stopped: bool,
    disable: bool,
) -> R<Harness<'static, State>> {
    app_height(mode, multi, wrap, short, stopped, disable, 100.)
}
fn app_height(
    mode: i32,
    multi: bool,
    wrap: bool,
    short: bool,
    stopped: bool,
    disable: bool,
    height: f32,
) -> R<Harness<'static, State>> {
    let mut ide = Ide::new();
    let mut export = String::from_utf8(fixture::export())?;
    if stopped {
        export.push_str("\nPrivate Sub Entry_MouseDown(ByVal b As Integer,ByVal s As Integer,ByVal x As Single,ByVal y As Single)\nDebug.Print \"down\"\nStop\nDebug.Print \"resumed\"\nEnd Sub\n");
    }
    if disable {
        export = export.replace(
            "Debug.Print \"down\"",
            "Entry.ScrollBars=0\nDebug.Print \"down\"",
        );
    }
    ide.import_native_form_export(
        "NativeForm.frm",
        export.as_bytes(),
        1252,
        &[NativeFormResource {
            name: "NativeForm.frx".into(),
            bytes: fixture::companion(),
        }],
    )?;
    let value = if short {
        "c.Text=\"short\""
    } else {
        "c.Text=\"AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA\"\nDim i As Long\nFor i=1 To 20\nc.Text=c.Text & ChrW(13) & ChrW(10) & \"Row \" & CStr(i) & \" BBBB WWWW\"\nNext i"
    };
    ide.add_module("Main.bas",&format!("Sub Main()\nDim c As MSForms.TextBox\nSet c=NativeForm.Entry\nNativeForm.Heading.Visible=False\nc.Left=16\nc.Top=16\nc.Width=180\nc.Height={height}\nc.FontSize=12\nc.ForeColor=RGB(201,7,23)\nc.MultiLine={multi}\nc.WordWrap={wrap}\nc.ScrollBars={mode}\nc.EnterFieldBehavior=1\nc.HideSelection=False\n{value}\nc.SelStart=1\nc.SelLength=3\nNativeForm.Apply.Top=140\nNativeForm.Show 0\nc.SetFocus\nEnd Sub"))?;
    ide.dispatch(Action::Run, &mut NoHost)?;
    for _ in 0..5000 {
        if ide.mode() != Mode::Running {
            break;
        }
        ide.tick(&mut NoHost)?;
    }
    assert_eq!(ide.mode(), Mode::Completed, "{:?}", ide.diagnostics);
    let mut h = Harness::builder()
        .with_size(egui::vec2(1100., 900.))
        .with_step_dt(1. / 60.)
        .allow_missing_glyphs()
        .wgpu()
        .build_ui_state(
            |ui, s: &mut State| {
                for command in s.view.show(ui, &s.ide, Theme::light()) {
                    if let Err(e) = apply(&mut s.ide, command, &mut NoHost) {
                        s.errors.push(e.to_string());
                    }
                }
                if let Err(e) = s.ide.tick(&mut NoHost) {
                    s.errors.push(e.to_string());
                }
            },
            State {
                ide,
                view: EditorUi::default(),
                errors: vec![],
            },
        );
    h.run_steps(30);
    if let Some(dir) = std::env::var_os("VBA_TEXT_SCROLL_SHOTS") {
        std::fs::create_dir_all(&dir)?;
        h.render()?.save(
            std::path::Path::new(&dir).join(format!("scroll-{mode}-{multi}-{wrap}-{short}.png")),
        )?;
    }
    Ok(h)
}
fn entry(h: &Harness<'_, State>) -> egui::Rect {
    h.get(
        egui_kittest::kittest::By::new()
            .role(egui::accesskit::Role::TextInput)
            .label("Entry"),
    )
    .rect()
}
fn bars(h: &Harness<'_, State>) -> Vec<egui::Rect> {
    let r = entry(h);
    h.query_all(egui_kittest::kittest::By::new().role(egui::accesskit::Role::ScrollBar))
        .map(|n| n.rect())
        .filter(|b| b.intersects(r))
        .collect()
}
fn pointer(h: &mut Harness<'_, State>, p: egui::Pos2, pressed: bool) {
    h.event(egui::Event::PointerMoved(p));
    h.event(egui::Event::PointerButton {
        pos: p,
        button: egui::PointerButton::Primary,
        pressed,
        modifiers: egui::Modifiers::NONE,
    });
}
fn field(h: &Harness<'_, State>, name: &str) -> R<Value> {
    let f = *h.state().ide.form_instances()?.first().ok_or("form")?;
    Ok(h.state().ide.form_instance_field(f, Some(12), name)?)
}
fn healthy(h: &Harness<'_, State>) {
    assert!(h.state().errors.is_empty(), "{:?}", h.state().errors);
}
fn pixels(h: &mut Harness<'_, State>) -> R<Vec<[u8; 4]>> {
    let r = entry(h).shrink(20.);
    Ok(h.render()?
        .enumerate_pixels()
        .filter(|(x, y, _)| r.contains(egui::pos2(*x as f32, *y as f32)))
        .map(|(_, _, p)| p.0)
        .collect())
}
#[test]
fn scrollbars_follow_overflow_axes_wordwrap_and_single_line_rules() -> R {
    for (mode, multi, wrap, short, count) in [
        (0, true, false, false, 0),
        (1, true, false, false, 1),
        (2, true, false, false, 1),
        (3, true, false, false, 2),
        (3, true, true, false, 1),
        (3, false, false, false, 1),
        (3, true, false, true, 0),
    ] {
        let h = app(mode, multi, wrap, short, false)?;
        let r = entry(&h);
        assert!(
            (r.width() - 240.).abs() < 2. && (r.height() - 133.333).abs() < 2.,
            "{r:?}"
        );
        assert_eq!(bars(&h).len(), count, "{mode}/{multi}/{wrap}/{short}");
        healthy(&h);
    }
    Ok(())
}
#[test]
fn dragging_vertical_scrollbar_changes_pixels_without_changing_text_range_or_events() -> R {
    let mut h = app(2, true, true, false, false)?;
    let before = pixels(&mut h)?;
    let text = field(&h, "VbaText")?;
    let bar = *bars(&h).first().ok_or("bar")?;
    let start = bar.center_top() + egui::vec2(0., 5.);
    let end = bar.center_bottom() - egui::vec2(0., 5.);
    pointer(&mut h, start, true);
    h.run_steps(25);
    h.event(egui::Event::PointerMoved(end));
    h.run_steps(25);
    pointer(&mut h, end, false);
    h.run_steps(30);
    let after = pixels(&mut h)?;
    if let Some(dir) = std::env::var_os("VBA_TEXT_SCROLL_SHOTS") {
        h.render()?
            .save(std::path::Path::new(&dir).join("after-drag.png"))?;
    }
    assert!(
        before
            .iter()
            .zip(after.iter())
            .filter(|(a, b)| a != b)
            .count()
            > 100,
        "scrolling must move rendered text independently of the caret"
    );
    assert_eq!(field(&h, "VbaText")?, text);
    assert_eq!(field(&h, "VbaSelStart")?, Value::Long(1));
    assert_eq!(field(&h, "VbaSelLength")?, Value::Long(3));
    assert_eq!(h.state().ide.output(), vec!["initialized"]);
    healthy(&h);
    Ok(())
}
#[test]
fn scrollbar_page_default_waits_for_stopped_mouse_down() -> R {
    let mut h = app(2, true, true, false, true)?;
    let before = pixels(&mut h)?;
    let bar = *bars(&h).first().ok_or("bar")?;
    let p = bar.center_bottom() - egui::vec2(0., 6.);
    pointer(&mut h, p, true);
    h.run_steps(25);
    assert_eq!(h.state().ide.mode(), Mode::Break);
    assert_eq!(field(&h, "VbaSelLength")?, Value::Long(3));
    h.state_mut().ide.dispatch(Action::Run, &mut NoHost)?;
    h.run_steps(35);
    pointer(&mut h, p, false);
    h.run_steps(30);
    assert!(
        before
            .iter()
            .zip(pixels(&mut h)?.iter())
            .filter(|(a, b)| a != b)
            .count()
            > 100,
        "page scrolling must move rendered text"
    );
    assert_eq!(
        h.state().ide.output(),
        vec!["initialized", "down", "resumed"]
    );
    healthy(&h);
    Ok(())
}
#[test]
fn horizontal_drag_and_scrolled_text_pointer_use_the_current_viewport() -> R {
    let mut h = app(1, true, false, false, false)?;
    let before = pixels(&mut h)?;
    let value = field(&h, "VbaText")?;
    let bar = *bars(&h).first().ok_or("horizontal bar")?;
    let start = bar.left_center() + egui::vec2(5., 0.);
    let end = bar.right_center() - egui::vec2(5., 0.);
    pointer(&mut h, start, true);
    h.run_steps(20);
    h.event(egui::Event::PointerMoved(end));
    h.run_steps(25);
    pointer(&mut h, end, false);
    h.run_steps(25);
    assert!(
        before
            .iter()
            .zip(pixels(&mut h)?.iter())
            .filter(|(a, b)| a != b)
            .count()
            > 100
    );
    assert_eq!(field(&h, "VbaText")?, value);
    assert_eq!(field(&h, "VbaSelLength")?, Value::Long(3));
    healthy(&h);
    // A separate vertical viewport must hit the displayed row, not the hidden first row.
    let mut h = app(2, true, true, false, false)?;
    let bar = *bars(&h).first().ok_or("vertical bar")?;
    pointer(&mut h, bar.center_top() + egui::vec2(0., 5.), true);
    h.run_steps(20);
    let end = bar.center_bottom() - egui::vec2(0., 5.);
    h.event(egui::Event::PointerMoved(end));
    h.run_steps(25);
    pointer(&mut h, end, false);
    h.run_steps(25);
    let pos = entry(&h).left_top() + egui::vec2(2., 8.);
    pointer(&mut h, pos, true);
    pointer(&mut h, pos, false);
    h.run_steps(45);
    let Value::Long(start) = field(&h, "VbaSelStart")? else {
        return Err("start".into());
    };
    let Value::Long(length) = field(&h, "VbaSelLength")? else {
        return Err("length".into());
    };
    assert!(start > 0 && length > 0, "{start}/{length}");
    healthy(&h);
    Ok(())
}
#[test]
fn programmatic_selection_scrolls_into_view_and_reset_discards_viewport_state() -> R {
    let mut h = app(2, true, true, false, false)?;
    let before = pixels(&mut h)?;
    let Value::String(value) = field(&h, "VbaText")? else {
        return Err("text".into());
    };
    let owner = *h.state().ide.form_instances()?.first().ok_or("owner")?;
    let end = u32::try_from(value.units().len())?;
    h.state_mut()
        .ide
        .form_instance_text(owner, 12, value.clone(), (end, 0), &mut NoHost)?;
    h.run_steps(65);
    let scrolled = pixels(&mut h)?;
    assert!(
        before
            .iter()
            .zip(scrolled.iter())
            .filter(|(a, b)| a != b)
            .count()
            > 100
    );
    assert_eq!(field(&h, "VbaText")?, Value::String(value));
    assert_eq!(field(&h, "VbaSelStart")?, Value::Long(i32::try_from(end)?));
    h.state_mut().ide.dispatch(Action::Reset, &mut NoHost)?;
    h.state_mut().ide.dispatch(Action::Run, &mut NoHost)?;
    h.run_steps(200);
    assert_eq!(
        h.state().ide.mode(),
        Mode::Completed,
        "{:?}",
        h.state().ide.diagnostics
    );
    assert_eq!(field(&h, "VbaSelStart")?, Value::Long(1));
    assert!(
        scrolled
            .iter()
            .zip(pixels(&mut h)?.iter())
            .filter(|(a, b)| a != b)
            .count()
            > 100
    );
    healthy(&h);
    Ok(())
}
#[test]
fn callback_policy_changes_remove_scrollbars_without_replaying_a_stale_default() -> R {
    let mut h = app_policy(2, true, true, false, true, true)?;
    let bar = *bars(&h).first().ok_or("bar")?;
    let p = bar.center_bottom() - egui::vec2(0., 6.);
    pointer(&mut h, p, true);
    h.run_steps(25);
    assert_eq!(h.state().ide.mode(), Mode::Break);
    assert_eq!(field(&h, "VbaScrollBars")?, Value::Long(0));
    pointer(&mut h, p, false);
    h.run_steps(5);
    h.state_mut().ide.dispatch(Action::Run, &mut NoHost)?;
    h.run_steps(80);
    assert!(bars(&h).is_empty());
    assert_eq!(field(&h, "VbaSelStart")?, Value::Long(1));
    assert_eq!(field(&h, "VbaSelLength")?, Value::Long(3));
    assert_eq!(
        h.state().ide.output(),
        vec!["initialized", "down", "resumed"]
    );
    healthy(&h);
    Ok(())
}
#[test]
fn a_horizontal_bar_requires_room_for_the_edit_line_as_well_as_the_bar() -> R {
    let h = app_height(1, false, false, false, false, false, 12.)?;
    assert!(bars(&h).is_empty());
    assert_eq!(field(&h, "VbaScrollBars")?, Value::Long(1));
    healthy(&h);
    Ok(())
}
#[test]
fn integrated_scrollbar_corner_does_not_change_the_text_selection() -> R {
    let mut h = app(3, true, false, false, false)?;
    assert_eq!(bars(&h).len(), 2);
    let p = entry(&h).right_bottom() - egui::vec2(3., 3.);
    pointer(&mut h, p, true);
    pointer(&mut h, p, false);
    h.run_steps(60);
    assert_eq!(field(&h, "VbaSelStart")?, Value::Long(1));
    assert_eq!(field(&h, "VbaSelLength")?, Value::Long(3));
    healthy(&h);
    Ok(())
}
