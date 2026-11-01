//! Attribute grammar validation; host-specific meanings are not execution support.
use crate::*;
use vba_diagnostics::Diagnostic;

pub fn attribute_integer(attribute: &Attribute) -> Option<i32> {
    let value = match &attribute.value.kind {
        ExprKind::Number(value) => value.parse::<i64>().ok()?,
        ExprKind::Unary { op, value } => {
            let ExprKind::Number(value) = &value.kind else {
                return None;
            };
            let value = value.parse::<i64>().ok()?;
            if op == "-" {
                value.checked_neg()?
            } else if op == "+" {
                value
            } else {
                return None;
            }
        }
        _ => return None,
    };
    i32::try_from(value).ok()
}

pub(crate) fn validate(module: &Module) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    let mut names = std::collections::BTreeSet::new();
    for attribute in &module.attributes {
        let key = attribute
            .path
            .last()
            .map(|name| name.key())
            .unwrap_or_default();
        let fail = |message: &str| Diagnostic::new("VBA0101", message).at(attribute.span);
        if matches!(attribute.owner, AttributeOwner::Module)
            && key != "vb_ext_key"
            && !names.insert(key.clone())
        {
            diagnostics.push(fail("duplicate module attribute"));
        }
        let string = matches!(attribute.value.kind, ExprKind::String(_));
        let boolean = matches!(attribute.value.kind, ExprKind::Boolean(_));
        let valid = match key.as_str() {
            "vb_name" => {
                matches!(&attribute.value.kind, ExprKind::String(name) if !name.is_empty() && name.chars().count() <= 31 && name.chars().enumerate().all(|(index,c)| c.is_alphabetic() || index > 0 && (c.is_alphanumeric() || c == '_')))
                    && matches!(attribute.owner, AttributeOwner::Module)
            }
            "vb_predeclaredid" | "vb_exposed" | "vb_creatable" | "vb_globalnamespace"
            | "vb_customizable" | "vb_templatederived" => {
                boolean && matches!(attribute.owner, AttributeOwner::Module)
            }
            "vb_description" | "vb_vardescription" | "vb_base" | "vb_invoke_func"
            | "vb_memberflags" | "vb_varmemberflags" => string,
            "vb_usermemid" | "vb_varusermemid" => {
                attribute_integer(attribute).is_some()
                    && matches!(attribute.owner, AttributeOwner::Member(_))
            }
            "vb_helpid" | "vb_varhelpid" => attribute_integer(attribute).is_some(),
            "vb_ext_key" => {
                string
                    && attribute.additional_values.len() == 1
                    && attribute
                        .additional_values
                        .iter()
                        .all(|value| matches!(value.kind, ExprKind::String(_)))
            }
            _ => true,
        };
        if !valid {
            diagnostics.push(fail("invalid attribute owner or literal type"));
        }
        if matches!(
            key.as_str(),
            "vb_name"
                | "vb_predeclaredid"
                | "vb_exposed"
                | "vb_creatable"
                | "vb_globalnamespace"
                | "vb_customizable"
                | "vb_templatederived"
                | "vb_description"
                | "vb_vardescription"
                | "vb_base"
                | "vb_invoke_func"
                | "vb_memberflags"
                | "vb_varmemberflags"
                | "vb_usermemid"
                | "vb_varusermemid"
                | "vb_helpid"
                | "vb_varhelpid"
        ) && !attribute.additional_values.is_empty()
        {
            diagnostics.push(fail("scalar attribute has multiple values"));
        }
        if diagnostics.len() >= MAX_DIAGNOSTICS {
            break;
        }
    }
    diagnostics
}
