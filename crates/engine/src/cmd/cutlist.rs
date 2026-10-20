//! Cut lists (UX prototype, issue #137). `cutlist.parts` lists the parts to cut with their stock
//! size, `cutlist.layout` places them on boards and sheets, and `cutlist.stock` keeps the stock you
//! can buy. Only the toolbar windows exist yet (`ui.start` opens them with sample data); run as
//! commands they say they are not built.

use serde_json::Value;

use super::CommandSpec;
use crate::{EngineError, Result, Session};

pub static COMMANDS: &[CommandSpec] = &[
    CommandSpec::new("cutlist.parts", "Cut List", not_built).at("SOLID", "INSPECT").icon("bom").noundo().params(
        "UX prototype: the toolbar opens the Cut List window with sample data; the command is not built yet (#137)",
    ),
    CommandSpec::new("cutlist.layout", "Layout on Stock", not_built).at("SOLID", "INSPECT").icon("sheet").noundo().params(
        "UX prototype: the toolbar opens the Layout on Stock window with sample data; the command is not built yet (#137)",
    ),
    CommandSpec::new("cutlist.stock", "Stock Library", not_built).at("SOLID", "INSPECT").icon("library").noundo().doc_only().params(
        "UX prototype: the toolbar opens the Stock Library window with sample data; the command is not built yet (#137)",
    ),
];

fn not_built(_: &mut Session, _: &Value) -> Result<Value> {
    Err(EngineError::Other("not supported yet: cut lists are a UX prototype (issue #137)".into()))
}
