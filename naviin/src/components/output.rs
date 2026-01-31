/// Output Component - Command results display area
///
/// Displays the output and results from executed commands.
/// Shows command history and responses in a scrollable format.
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::Stylize,
    symbols::border,
    text::{Line, Text},
    widgets::{Block, Paragraph, Widget},
};

/// Component for displaying command output and results
pub struct OutputComponent {
    /// The current output text to display
    output_text: String,
    /// History of previous outputs
    history: Vec<String>,
}

impl Default for OutputComponent {
    fn default() -> Self {
        Self::new()
    }
}

impl OutputComponent {
    /// SECTION: Constructor

    /// Creates a new output component with empty content
    pub fn new() -> Self {
        Self {
            output_text: String::new(),
            history: Vec::new(),
        }
    }

    /// SECTION: Output Management

    /// Sets the current output text to display
    ///
    /// # Arguments
    /// * `text` - The output text to show
    pub fn set_output(&mut self, text: String) {
        self.output_text = text;
    }

    /// Appends text to the current output
    ///
    /// # Arguments
    /// * `text` - Text to append
    pub fn append_output(&mut self, text: &str) {
        if !self.output_text.is_empty() {
            self.output_text.push('\n');
        }
        self.output_text.push_str(text);
    }

    /// Adds current output to history and clears display
    pub fn commit_to_history(&mut self) {
        if !self.output_text.is_empty() {
            self.history.push(self.output_text.clone());
        }
    }

    /// Clears the current output display
    pub fn clear(&mut self) {
        self.output_text.clear();
    }

    /// SECTION: Query Methods

    /// Returns the current output text
    pub fn get_output(&self) -> &str {
        &self.output_text
    }

    /// Returns the full output history
    pub fn get_history(&self) -> &[String] {
        &self.history
    }
}

impl Widget for &OutputComponent {
    /// Renders the output area with the current output text
    fn render(self, area: Rect, buf: &mut Buffer) {
        let text = if self.output_text.is_empty() {
            Text::from(vec![
                Line::from(""),
                Line::from("Command output will appear here")
                    .centered()
                    .dim(),
            ])
        } else {
            Text::from(self.output_text.clone())
        };

        let block = Block::bordered()
            .title(" Output ".bold())
            .border_set(border::ROUNDED);

        Paragraph::new(text).block(block).render(area, buf);
    }
}
