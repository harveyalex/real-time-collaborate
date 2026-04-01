use spacetimedb_lib::SpacetimeType;
use serde::{Serialize, Deserialize};

#[derive(SpacetimeType, Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[sats(crate = spacetimedb_lib)]
pub enum ElementKind {
    Rectangle,
    Ellipse,
    Arrow,
    Line,
    Text,
    Freehand,
}

#[derive(SpacetimeType, Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[sats(crate = spacetimedb_lib)]
pub enum UndoAction {
    Create,
    Update,
    Delete,
}

/// Default styling for new elements.
pub const DEFAULT_STROKE_COLOR: u32 = 0xFFFFFFFF; // white
pub const DEFAULT_FILL_COLOR: u32 = 0x00000000;   // transparent
pub const DEFAULT_STROKE_WIDTH: f32 = 2.0;
pub const DEFAULT_OPACITY: f32 = 1.0;
pub const DEFAULT_FONT_SIZE: f32 = 20.0;

#[cfg(test)]
mod tests {
    use super::*;
    use spacetimedb_lib::bsatn;

    #[test]
    fn element_kind_round_trip() {
        let kinds = [
            ElementKind::Rectangle,
            ElementKind::Ellipse,
            ElementKind::Arrow,
            ElementKind::Line,
            ElementKind::Text,
            ElementKind::Freehand,
        ];
        for kind in kinds {
            let encoded = bsatn::to_vec(&kind).unwrap();
            let decoded: ElementKind = bsatn::from_slice(&encoded).unwrap();
            assert_eq!(kind, decoded);
        }
    }

    #[test]
    fn undo_action_round_trip() {
        let actions = [
            UndoAction::Create,
            UndoAction::Update,
            UndoAction::Delete,
        ];
        for action in actions {
            let encoded = bsatn::to_vec(&action).unwrap();
            let decoded: UndoAction = bsatn::from_slice(&encoded).unwrap();
            assert_eq!(action, decoded);
        }
    }
}
