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
    /// Current scroll offset (how many lines scrolled down)
    scroll_offset: usize,
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
            scroll_offset: 0,
        }
    }

    /// SECTION: Output Management

    /// Sets the current output text to display
    /// Resets scroll position to show the beginning of new content
    ///
    /// # Arguments
    /// * `text` - The output text to show
    pub fn set_output(&mut self, text: String) {
        self.output_text = text;
        self.reset_scroll();
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

    /// SECTION: Scrolling

    /// Scrolls the output up by the specified number of lines
    ///
    /// # Arguments
    /// * `lines` - Number of lines to scroll up
    pub fn scroll_up(&mut self, lines: usize) {
        self.scroll_offset = self.scroll_offset.saturating_sub(lines);
    }

    /// Scrolls the output down by the specified number of lines
    ///
    /// # Arguments
    /// * `lines` - Number of lines to scroll down
    pub fn scroll_down(&mut self, lines: usize) {
        self.scroll_offset += lines;
    }

    /// Scrolls to the top of the output
    pub fn scroll_to_top(&mut self) {
        self.scroll_offset = 0;
    }

    /// Scrolls to the bottom of the output
    pub fn scroll_to_bottom(&mut self) {
        // Set to a large number - widget will clamp to actual max
        self.scroll_offset = usize::MAX;
    }

    /// Resets scroll offset when new content is displayed
    fn reset_scroll(&mut self) {
        self.scroll_offset = 0;
    }
}

impl Widget for &OutputComponent {
    /// Renders the output area with the current output text
    /// Applies scrolling based on the current scroll offset
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
            .title_bottom(
                Line::from(vec![
                    " Scroll ".into(),
                    "<PgUp>/<PgDn>".blue().bold(),
                    " Top ".into(),
                    "<Ctrl+Home>".blue().bold(),
                    " Bottom ".into(),
                    "<Ctrl+End>".blue().bold(),
                ])
                .centered(),
            )
            .border_set(border::ROUNDED);

        // Calculate visible area (accounting for borders)
        let inner_area = block.inner(area);
        let visible_lines = inner_area.height as usize;

        // Only apply scroll if content exceeds visible area
        let total_lines = self.output_text.lines().count();
        let scroll = if total_lines > visible_lines {
            (self.scroll_offset as u16).min((total_lines - visible_lines) as u16)
        } else {
            0
        };

        Paragraph::new(text)
            .block(block)
            .scroll((scroll, 0))
            .render(area, buf);
    }
}
