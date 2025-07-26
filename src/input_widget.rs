use ratatui::{
    style::{Color, Style, Stylize},
    widgets::{Block, Paragraph, Widget},
};
use tui_input::Input;

pub struct InputWidget<'a> {
    pub is_focused: bool,
    pub input: &'a Input,
    pub title: &'a str,
}
impl<'a> Widget for &InputWidget<'a> {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        let width = area.width - 3;
        let scroll = self.input.visual_scroll(width as usize);
        let input_style = if self.is_focused {
            Color::Yellow.into()
        } else {
            Style::default()
        };
        Paragraph::new(if self.is_focused && self.input.value().is_empty() {
            "Type Here\u{2588}".italic().dark_gray()
        } else if self.is_focused {
            format!("{}\u{2588}", self.input.value()).into()
        } else {
            self.input.value().into()
        })
        .style(input_style)
        .scroll((0, scroll as u16))
        .block(Block::bordered().title(format!(" {title} ", title = self.title)))
        .render(area, buf);
    }
}
