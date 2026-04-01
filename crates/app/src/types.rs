/// Pure enums used by both the vim state machine and app state.
/// No WASM/Leptos dependencies — safe to compile in any target.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VimMode {
    Normal,
    Insert,
    Visual,
    Command,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tool {
    Select,
    Rectangle,
    Ellipse,
    Arrow,
    Line,
    Text,
    Freehand,
}
