//! Original headless designer workflows; no Office binaries or fixtures.
use vba_ide::forms::{Align, Bounds, ControlKind, DesignEdit, Form, FormDesigner, SameSize};
type R = Result<(), Box<dyn std::error::Error>>;
fn bounds(left: f32, top: f32) -> Bounds {
    Bounds {
        left,
        top,
        width: 60.0,
        height: 24.0,
    }
}
#[test]
fn controls_properties_geometry_and_tab_order_remain_independent() -> R {
    let mut d = FormDesigner::new(Form::new("CustomerForm")?)?;
    d.edit(DesignEdit::Add(ControlKind::Label, bounds(8.0, 8.0)))?;
    d.edit(DesignEdit::Add(ControlKind::TextBox, bounds(79.0, 8.0)))?;
    d.edit(DesignEdit::Add(
        ControlKind::CommandButton,
        bounds(8.0, 80.0),
    ))?;
    assert_eq!(d.form().controls.first().ok_or("label")?.bounds.left, 6.0);
    assert!(!d.form().controls.first().ok_or("label")?.tab_stop);
    let order = d.form().tab_order.clone();
    let first = *order.first().ok_or("order")?;
    d.select(Some(first), false)?;
    d.edit(DesignEdit::Front)?;
    assert_eq!(d.form().tab_order, order);
    assert_eq!(d.form().controls.last().ok_or("front")?.id, first);
    let mut reverse = order;
    reverse.reverse();
    d.edit(DesignEdit::TabOrder(reverse.clone()))?;
    d.select(None, false)?;
    d.tab(false);
    assert!(d.selection().contains(reverse.first().ok_or("order")?));
    d.tab(true);
    assert!(d.selection().contains(reverse.last().ok_or("order")?));
    Ok(())
}
#[test]
fn group_selection_transforms_alignment_and_history_are_atomic() -> R {
    let mut d = FormDesigner::new(Form::new("Layout")?)?;
    d.set_grid(6.0, false)?;
    d.edit(DesignEdit::Add(ControlKind::TextBox, bounds(7.0, 11.0)))?;
    d.edit(DesignEdit::Add(
        ControlKind::CommandButton,
        bounds(77.0, 41.0),
    ))?;
    d.select(Some(1), false)?;
    d.select(Some(2), true)?;
    d.edit(DesignEdit::Align(Align::Top))?;
    assert!(d.form().controls.iter().all(|c| c.bounds.top == 41.0));
    d.edit(DesignEdit::Group)?;
    d.select(None, false)?;
    d.select(Some(1), false)?;
    assert_eq!(d.selection().len(), 2);
    let before = d.form().clone();
    assert!(d.edit(DesignEdit::Resize(-100.0, 0.0)).is_err());
    assert_eq!(d.form(), &before);
    d.edit(DesignEdit::Move(6.0, 12.0))?;
    d.undo()?;
    assert_eq!(d.form(), &before);
    d.redo()?;
    assert_ne!(d.form(), &before);
    d.edit(DesignEdit::Ungroup)?;
    d.select(Some(1), false)?;
    assert_eq!(d.selection().len(), 1);
    Ok(())
}
#[test]
fn formatting_spacing_and_centering_preserve_independent_tab_order() -> R {
    let mut d = FormDesigner::new(Form::new("Spacing")?)?;
    d.set_grid(6.0, false)?;
    for x in [0.0, 90.0, 240.0] {
        d.edit(DesignEdit::Add(ControlKind::CommandButton, bounds(x, 0.0)))?;
    }
    d.select(Some(1), false)?;
    d.select(Some(2), true)?;
    d.select(Some(3), true)?;
    d.edit(DesignEdit::Space { horizontal: true })?;
    assert_eq!(d.form().controls.get(1).ok_or("middle")?.bounds.left, 120.0);
    d.edit(DesignEdit::SameSize(SameSize::Both))?;
    d.edit(DesignEdit::Center { horizontal: false })?;
    assert!(d.form().controls.iter().all(|c| c.bounds.top == 88.0));
    assert_eq!(d.form().tab_order, [1, 2, 3]);
    Ok(())
}
#[test]
fn invalid_properties_names_order_and_limits_never_replace_design() -> R {
    let mut d = FormDesigner::new(Form::new("Validation")?)?;
    d.edit(DesignEdit::Add(ControlKind::TextBox, bounds(0.0, 0.0)))?;
    let original = d.form().clone();
    let mut c = original.controls.first().ok_or("control")?.clone();
    c.name = "End".into();
    assert!(d.edit(DesignEdit::Replace(c)).is_err());
    assert_eq!(d.form(), &original);
    assert!(d.edit(DesignEdit::TabOrder(vec![999])).is_err());
    assert_eq!(d.form(), &original);
    assert!(d.edit(DesignEdit::Move(f32::NAN, 0.0)).is_err());
    assert!(
        d.edit(DesignEdit::FormProperties {
            caption: String::new(),
            width: f32::INFINITY,
            height: 200.0
        })
        .is_err()
    );
    assert!(d.set_zoom(401).is_err());
    assert_eq!(d.zoom(), 100);
    assert!(d.set_grid(0.0, true).is_err());
    assert_eq!(d.grid(), 6.0);
    for _ in 1..128 {
        d.edit(DesignEdit::Add(ControlKind::Label, bounds(0.0, 0.0)))?;
    }
    assert!(
        d.edit(DesignEdit::Add(ControlKind::Label, bounds(0.0, 0.0)))
            .is_err()
    );
    assert_eq!(d.form().controls.len(), 128);
    Ok(())
}
#[test]
fn deleting_group_then_undo_restores_controls_and_does_not_recycle_ids() -> R {
    let mut d = FormDesigner::new(Form::new("Lifetime")?)?;
    d.edit(DesignEdit::Add(ControlKind::Label, bounds(0.0, 0.0)))?;
    d.edit(DesignEdit::Add(ControlKind::Label, bounds(0.0, 30.0)))?;
    d.select(Some(1), false)?;
    d.select(Some(2), true)?;
    d.edit(DesignEdit::Group)?;
    d.edit(DesignEdit::Delete)?;
    assert!(d.form().controls.is_empty());
    d.undo()?;
    assert_eq!(d.form().controls.len(), 2);
    d.edit(DesignEdit::Add(ControlKind::Label, bounds(0.0, 60.0)))?;
    assert_eq!(d.form().controls.last().ok_or("new")?.id, 3);
    Ok(())
}

#[test]
fn saved_design_identity_and_snap_survive_undo_and_zoom_is_view_state() -> R {
    let mut d = FormDesigner::new(Form::new("SavedDesign")?)?;
    assert!(!d.dirty());
    d.edit(DesignEdit::Add(ControlKind::TextBox, bounds(0.0, 0.0)))?;
    d.mark_saved();
    let saved = d.form().clone();
    d.set_zoom(200)?;
    assert!(!d.dirty());
    d.edit(DesignEdit::Move(8.0, 8.0))?;
    assert_eq!(d.form().controls.first().ok_or("control")?.bounds.left, 6.0);
    assert!(d.dirty());
    assert!(d.can_undo());
    d.undo()?;
    assert_eq!(d.form(), &saved);
    assert!(!d.dirty());
    assert!(d.can_redo());
    assert_eq!(d.zoom(), 200);
    d.redo()?;
    assert!(d.dirty());
    Ok(())
}
