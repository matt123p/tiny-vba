//! Inert native Office UserForm data. No filesystem, control activation or executable cache.
mod binary;
mod frame;
use crate::{Attribute, Limits, Metered, Result, budget, container_error, read_stream};
pub use frame::{companion_name, decode_export};
use std::io::Cursor;

pub const CAPABILITY: &str = "native-userform-import-v1";
pub const MAX_CONTROLS: usize = 128;
pub const MAX_RESOURCE_BYTES: usize = 4 * 1024 * 1024;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Format {
    Export,
    Embedded,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Label,
    TextBox,
    CommandButton,
    CheckBox,
    OptionButton,
    ToggleButton,
    ScrollBar,
    SpinButton,
}
/// Coordinates are original signed HIMETRIC units, not pixels or points.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Control {
    pub id: u32,
    pub name: String,
    pub kind: Kind,
    pub left: i32,
    pub top: i32,
    pub width: i32,
    pub height: i32,
    pub caption: String,
    pub text: String,
    pub enabled: bool,
    pub visible: bool,
    pub tab_stop: bool,
    pub tab_index: i16,
    pub default: bool,
    pub cancel: bool,
    pub max_length: i32,
    pub group_name: String,
    pub locked: bool,
    pub text_behavior: TextBehavior,
    pub numeric: Option<NumericBehavior>,
}
/// Native SpinButton/ScrollBar state, without executing control resources.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NumericBehavior {
    pub minimum: i32,
    pub maximum: i32,
    pub value: i32,
    pub small_change: i32,
    pub large_change: i32,
    pub orientation: i32,
    pub delay: i32,
}
/// Selected native MorphData behavior; original resources remain separate.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TextBehavior {
    pub multi_line: bool,
    pub word_wrap: bool,
    pub enter_key: bool,
    pub tab_key: bool,
    pub recall_selection: bool,
    pub password_char: u16,
}
impl Default for TextBehavior {
    fn default() -> Self {
        Self {
            multi_line: false,
            word_wrap: true,
            enter_key: false,
            tab_key: false,
            recall_selection: false,
            password_char: 0,
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Design {
    pub caption: String,
    pub width: i32,
    pub height: i32,
    /// Native site/painting order. Only immediate children are admitted in v1.
    pub controls: Vec<Control>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Resource {
    /// Original stream path or caller-supplied companion basename. Never a path to open.
    pub name: String,
    pub bytes: Vec<u8>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NativeForm {
    pub format: Format,
    pub name: String,
    pub code_page: u16,
    /// Exact decoded authored code including attributes and line endings, without designer header.
    pub source: String,
    pub attributes: Vec<Attribute>,
    pub design: Design,
    /// Original designer/frame and companion streams, independent of mutable IDE state.
    pub resources: Vec<Resource>,
    /// Retained design/formatting properties the initial shared renderer does not interpret.
    pub diagnostics: Vec<String>,
}
pub(crate) fn open_resource<'a>(
    bytes: &'a [u8],
    limits: &Limits,
) -> Result<cfb::CompoundFile<Metered<'a>>> {
    if bytes.len() > MAX_RESOURCE_BYTES.min(limits.input_bytes) {
        return Err(budget());
    }
    let sector = if bytes.get(30..32) == Some(&[12, 0]) {
        4096
    } else {
        512
    };
    if !bytes.len().is_multiple_of(sector) {
        return Err(crate::corrupt("partial native resource CFB sector"));
    }
    let reader = Metered {
        inner: Cursor::new(bytes),
        reads: bytes.len().saturating_mul(16),
        operations: 200_000,
    };
    let file = cfb::CompoundFile::open_strict(reader).map_err(container_error)?;
    let mut count = 0;
    for e in file.walk() {
        count += 1;
        if count > 512.min(limits.entries)
            || e.path().components().count() > 4
            || e.len() > MAX_RESOURCE_BYTES.min(limits.stream_bytes) as u64
        {
            return Err(budget());
        }
    }
    Ok(file)
}
pub(crate) fn embedded(
    file: &mut cfb::CompoundFile<Metered<'_>>,
    storage: &str,
    name: &str,
    source: &str,
    cp: u16,
    limits: &Limits,
) -> Result<NativeForm> {
    let root = format!("/{storage}");
    let frame_name = format!("{root}/\u{3}VBFrame");
    let raw = read_stream(
        file,
        &frame_name,
        limits.stream_bytes.min(MAX_RESOURCE_BYTES),
    )?;
    let text = crate::text::decode(&raw, cp)?;
    let header = frame::parse(&text, false)?;
    if header.name != name {
        return Err(crate::corrupt("designer/source identity disagreement"));
    }
    let mut resources = vec![Resource {
        name: frame_name,
        bytes: raw,
    }];
    let mut total = resources.first().map_or(0, |r| r.bytes.len());
    let paths: Vec<_> = file
        .walk_storage(&root)
        .map_err(container_error)?
        .filter(|e| e.is_stream())
        .map(|e| e.path().to_owned())
        .collect();
    for path in paths {
        let path = path
            .to_str()
            .ok_or_else(|| crate::corrupt("designer stream path"))?;
        if path.ends_with("/\u{3}VBFrame") {
            continue;
        }
        let bytes = read_stream(
            file,
            path,
            limits
                .stream_bytes
                .min(MAX_RESOURCE_BYTES.saturating_sub(total)),
        )?;
        total = total
            .checked_add(bytes.len())
            .filter(|n| *n <= MAX_RESOURCE_BYTES)
            .ok_or_else(budget)?;
        resources.push(Resource {
            name: path.into(),
            bytes,
        });
    }
    let mut result = binary::decode(&resources, &root, &header)?;
    result.format = Format::Embedded;
    result.code_page = cp;
    result.source = source.into();
    result.attributes = crate::text::attributes(source, name)?;
    result.resources = resources;
    Ok(result)
}
