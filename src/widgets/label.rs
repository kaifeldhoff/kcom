use tui::buffer::Buffer;
use tui::layout::Rect;
use tui::style::Style;
use tui::widgets::Widget;

pub struct Label {
    lines: Vec<String>,
    style: Style,
}

impl Default for Label {
    fn default() -> Label {
        Label {
            lines: vec![],
            style: Style::default(),
        }
    }
}

impl Widget for Label {
    fn draw(&mut self, area: Rect, buf: &mut Buffer) {
        for (i, line) in self.lines.iter().enumerate() {
            buf.set_string(area.left(), area.top() + (i as u16), line, self.style);
        }
    }
}

impl Label {
    pub fn text(&mut self, lines: Vec<String>) -> &mut Label {
        self.lines = lines;
        self
    }
    pub fn style(mut self, style: Style) -> Label {
        self.style = style;
        self
    }
}
