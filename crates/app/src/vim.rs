use crate::types::{VimMode, Tool};

#[derive(Debug, PartialEq)]
pub enum VimAction {
    None,
    SetTool(Tool),
    MoveSelected(f64, f64),
    DeleteSelected,
    CopySelected,
    Paste,
    Undo,
    Redo,
    EnterCommand,
    EnterVisual,
    ExitToNormal,
    CommandChar(char),
    CommandSubmit(String),
    CommandBackspace,
    ToggleHelp,
}

pub struct VimStateMachine {
    mode: VimMode,
    key_buffer: String,
    command_buffer: String,
}

impl VimStateMachine {
    pub fn new() -> Self {
        Self {
            mode: VimMode::Normal,
            key_buffer: String::new(),
            command_buffer: String::new(),
        }
    }

    pub fn mode(&self) -> VimMode {
        self.mode
    }

    pub fn key_buffer(&self) -> &str {
        &self.key_buffer
    }

    pub fn command_buffer(&self) -> &str {
        &self.command_buffer
    }

    pub fn handle_key(&mut self, key: &str, shift: bool, ctrl: bool) -> VimAction {
        match self.mode {
            VimMode::Normal => self.handle_normal(key, shift, ctrl),
            VimMode::Insert => self.handle_insert(key, shift, ctrl),
            VimMode::Visual => self.handle_visual(key, shift, ctrl),
            VimMode::Command => self.handle_command(key, shift, ctrl),
        }
    }

    fn handle_normal(&mut self, key: &str, shift: bool, ctrl: bool) -> VimAction {
        // Check ctrl+r first (Redo)
        if ctrl && key == "r" {
            self.key_buffer.clear();
            return VimAction::Redo;
        }

        let step = if shift { 1.0_f64 } else { 10.0_f64 };

        match key {
            "h" => {
                self.key_buffer.clear();
                VimAction::MoveSelected(-step, 0.0)
            }
            "j" => {
                self.key_buffer.clear();
                VimAction::MoveSelected(0.0, step)
            }
            "k" => {
                self.key_buffer.clear();
                VimAction::MoveSelected(0.0, -step)
            }
            "l" => {
                self.key_buffer.clear();
                VimAction::MoveSelected(step, 0.0)
            }
            "r" => {
                // ctrl is false here (checked above)
                self.key_buffer.clear();
                self.mode = VimMode::Insert;
                VimAction::SetTool(Tool::Rectangle)
            }
            "e" => {
                self.key_buffer.clear();
                self.mode = VimMode::Insert;
                VimAction::SetTool(Tool::Ellipse)
            }
            "a" => {
                self.key_buffer.clear();
                self.mode = VimMode::Insert;
                VimAction::SetTool(Tool::Arrow)
            }
            "L" => {
                // shift+l sends key="L"
                self.key_buffer.clear();
                self.mode = VimMode::Insert;
                VimAction::SetTool(Tool::Line)
            }
            "t" => {
                self.key_buffer.clear();
                self.mode = VimMode::Insert;
                VimAction::SetTool(Tool::Text)
            }
            "f" => {
                self.key_buffer.clear();
                self.mode = VimMode::Insert;
                VimAction::SetTool(Tool::Freehand)
            }
            "d" => {
                if self.key_buffer == "d" {
                    self.key_buffer.clear();
                    VimAction::DeleteSelected
                } else {
                    self.key_buffer = "d".to_string();
                    VimAction::None
                }
            }
            "y" => {
                if self.key_buffer == "y" {
                    self.key_buffer.clear();
                    VimAction::CopySelected
                } else {
                    self.key_buffer = "y".to_string();
                    VimAction::None
                }
            }
            "p" => {
                self.key_buffer.clear();
                VimAction::Paste
            }
            "u" => {
                self.key_buffer.clear();
                VimAction::Undo
            }
            "v" => {
                self.key_buffer.clear();
                self.mode = VimMode::Visual;
                VimAction::EnterVisual
            }
            ":" => {
                self.key_buffer.clear();
                self.command_buffer.clear();
                self.mode = VimMode::Command;
                VimAction::EnterCommand
            }
            "?" => VimAction::ToggleHelp,
            "Escape" => {
                self.key_buffer.clear();
                VimAction::ExitToNormal
            }
            _ => VimAction::None,
        }
    }

    fn handle_insert(&mut self, key: &str, _shift: bool, _ctrl: bool) -> VimAction {
        match key {
            "Escape" => {
                self.mode = VimMode::Normal;
                VimAction::SetTool(Tool::Select)
            }
            _ => VimAction::None,
        }
    }

    fn handle_visual(&mut self, key: &str, shift: bool, _ctrl: bool) -> VimAction {
        let step = if shift { 1.0_f64 } else { 10.0_f64 };

        match key {
            "h" => VimAction::MoveSelected(-step, 0.0),
            "j" => VimAction::MoveSelected(0.0, step),
            "k" => VimAction::MoveSelected(0.0, -step),
            "l" => VimAction::MoveSelected(step, 0.0),
            "d" => {
                self.mode = VimMode::Normal;
                VimAction::DeleteSelected
            }
            "y" => {
                self.mode = VimMode::Normal;
                VimAction::CopySelected
            }
            "Escape" => {
                self.mode = VimMode::Normal;
                VimAction::ExitToNormal
            }
            _ => VimAction::None,
        }
    }

    fn handle_command(&mut self, key: &str, _shift: bool, _ctrl: bool) -> VimAction {
        match key {
            "Escape" => {
                self.mode = VimMode::Normal;
                self.command_buffer.clear();
                VimAction::ExitToNormal
            }
            "Enter" => {
                let cmd = self.command_buffer.clone();
                self.command_buffer.clear();
                self.mode = VimMode::Normal;
                VimAction::CommandSubmit(cmd)
            }
            "Backspace" => {
                self.command_buffer.pop();
                VimAction::CommandBackspace
            }
            _ => {
                // Single char input
                if key.len() == 1 {
                    let ch = key.chars().next().unwrap();
                    self.command_buffer.push(ch);
                    VimAction::CommandChar(ch)
                } else {
                    VimAction::None
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normal_to_insert_via_r() {
        let mut vim = VimStateMachine::new();
        assert_eq!(vim.mode(), VimMode::Normal);
        let action = vim.handle_key("r", false, false);
        assert_eq!(vim.mode(), VimMode::Insert);
        assert!(matches!(action, VimAction::SetTool(Tool::Rectangle)));
    }

    #[test]
    fn insert_to_normal_via_escape() {
        let mut vim = VimStateMachine::new();
        vim.handle_key("r", false, false);
        let action = vim.handle_key("Escape", false, false);
        assert_eq!(vim.mode(), VimMode::Normal);
        assert!(matches!(action, VimAction::SetTool(Tool::Select)));
    }

    #[test]
    fn normal_dd_deletes() {
        let mut vim = VimStateMachine::new();
        let a1 = vim.handle_key("d", false, false);
        assert!(matches!(a1, VimAction::None));
        let a2 = vim.handle_key("d", false, false);
        assert!(matches!(a2, VimAction::DeleteSelected));
    }

    #[test]
    fn normal_to_command_mode() {
        let mut vim = VimStateMachine::new();
        let action = vim.handle_key(":", false, false);
        assert_eq!(vim.mode(), VimMode::Command);
        assert!(matches!(action, VimAction::EnterCommand));
    }

    #[test]
    fn hjkl_movement() {
        let mut vim = VimStateMachine::new();
        assert!(matches!(vim.handle_key("h", false, false), VimAction::MoveSelected(dx, _) if dx == -10.0));
        assert!(matches!(vim.handle_key("j", false, false), VimAction::MoveSelected(_, dy) if dy == 10.0));
        assert!(matches!(vim.handle_key("k", false, false), VimAction::MoveSelected(_, dy) if dy == -10.0));
        assert!(matches!(vim.handle_key("l", false, false), VimAction::MoveSelected(dx, _) if dx == 10.0));
    }

    #[test]
    fn shift_hjkl_fine_movement() {
        let mut vim = VimStateMachine::new();
        assert!(matches!(vim.handle_key("h", true, false), VimAction::MoveSelected(dx, _) if dx == -1.0));
    }

    #[test]
    fn undo_redo() {
        let mut vim = VimStateMachine::new();
        assert!(matches!(vim.handle_key("u", false, false), VimAction::Undo));
        assert!(matches!(vim.handle_key("r", false, true), VimAction::Redo));
    }

    #[test]
    fn visual_mode() {
        let mut vim = VimStateMachine::new();
        let a = vim.handle_key("v", false, false);
        assert_eq!(vim.mode(), VimMode::Visual);
        assert!(matches!(a, VimAction::EnterVisual));
    }

    #[test]
    fn command_submit() {
        let mut vim = VimStateMachine::new();
        vim.handle_key(":", false, false);
        vim.handle_key("w", false, false);
        let a = vim.handle_key("Enter", false, false);
        assert_eq!(vim.mode(), VimMode::Normal);
        assert!(matches!(a, VimAction::CommandSubmit(ref s) if s == "w"));
    }

    #[test]
    fn command_backspace() {
        let mut vim = VimStateMachine::new();
        vim.handle_key(":", false, false);
        vim.handle_key("w", false, false);
        vim.handle_key("q", false, false);
        vim.handle_key("Backspace", false, false);
        assert_eq!(vim.command_buffer(), "w");
    }
}
