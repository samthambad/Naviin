/// Input Component - Command typing area
///
/// Handles user text input with cursor navigation.
/// Provides a text field where users can type commands.
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::Stylize,
    symbols::border,
    text::{Line, Text},
    widgets::{Block, Paragraph, Widget},
};

/// Component for handling command input from the user
pub struct InputComponent {
    /// The current command text being typed
    command: String,
    /// Current cursor position (character index)
    cursor_position: usize,
}

impl Default for InputComponent {
    fn default() -> Self {
        Self::new()
    }
}

impl InputComponent {
    /// SECTION: Constructor

    /// Creates a new input component with empty command
    pub fn new() -> Self {
        Self {
            command: String::new(),
            cursor_position: 0,
        }
    }

    /// SECTION: Input Handling

    /// Adds a character at the current cursor position
    /// Moves cursor right after insertion
    ///
    /// # Arguments
    /// * `ch` - Character to insert
    pub fn enter_char(&mut self, ch: char) {
        let index = self.byte_index();
        self.command.insert(index, ch);
        self.move_cursor_right();
    }

    /// Removes the character before the cursor (backspace)
    pub fn backspace(&mut self) {
        if self.cursor_position > 0 {
            self.move_cursor_left();
            let index = self.byte_index();
            self.command.remove(index);
        }
    }

    /// SECTION: Cursor Navigation

    /// Moves cursor one position to the left
    /// Stops at the beginning of the text
    pub fn move_cursor_left(&mut self) {
        let new_pos = self.cursor_position.saturating_sub(1);
        self.cursor_position = self.clamp_cursor(new_pos);
    }

    /// Moves cursor one position to the right
    /// Stops at the end of the text
    pub fn move_cursor_right(&mut self) {
        let new_pos = self.cursor_position.saturating_add(1);
        self.cursor_position = self.clamp_cursor(new_pos);
    }

    /// Moves cursor to the beginning of the command
    pub fn move_cursor_start(&mut self) {
        self.cursor_position = 0;
    }

    /// Moves cursor to the end of the command
    pub fn move_cursor_end(&mut self) {
        self.cursor_position = self.command.len();
    }

    /// SECTION: Query Methods

    /// Returns the current command text
    pub fn get_command(&self) -> &str {
        &self.command
    }

    /// Clears the current command and resets cursor
    pub fn clear(&mut self) {
        self.command.clear();
        self.cursor_position = 0;
    }

    /// SECTION: Helper Methods

    /// Converts character index to byte index for string operations
    fn byte_index(&self) -> usize {
        self.cursor_position
    }

    /// Ensures cursor position stays within valid bounds
    fn clamp_cursor(&self, new_cursor_pos: usize) -> usize {
        new_cursor_pos.clamp(0, self.command.len())
    }
}

impl Widget for &InputComponent {
    /// Renders the input area with the command text and cursor
    fn render(self, area: Rect, buf: &mut Buffer) {
        let text = Text::from(Line::from(self.command.clone()));

        let block = Block::bordered()
            .title(" Command ".bold())
            .border_set(border::ROUNDED);

        Paragraph::new(text).block(block).render(area, buf);
    }
}
