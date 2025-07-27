use ratatui::{
    crossterm::event::{Event, KeyCode, KeyEvent},
    widgets::{Block, Paragraph},
};

use crate::WindowActionResult;

const HELP_INFO: &str = r#"
this is a help page.
It is very helpful!
"#;

pub struct HelpWindow {}
impl HelpWindow {
    pub fn new() -> Self {
        Self {}
    }

    pub(crate) fn draw(
        &self,
        state: &crate::state::State,
        frame: &mut ratatui::Frame<'_>,
        main_area: ratatui::prelude::Rect,
    ) {
        frame.render_widget(
            Paragraph::new(HELP_INFO)
                .centered()
                .block(Block::bordered().title(" Help ")),
            main_area,
        );
    }

    pub(crate) fn handle_event(
        &self,
        state: &mut crate::state::State,
        event: &ratatui::crossterm::event::Event,
    ) -> WindowActionResult {
        match event {
            Event::Key(KeyEvent {
                code: KeyCode::Char('1'),
                ..
            }) => {
                return WindowActionResult::FirstWindow;
            }
            Event::Key(KeyEvent {
                code: KeyCode::Char('2'),
                ..
            }) => {
                return WindowActionResult::SecondWindow;
            }
            Event::Key(KeyEvent {
                code: KeyCode::Char('3'),
                ..
            }) => {
                return WindowActionResult::ThirdWindow;
            }
            Event::Key(KeyEvent {
                code: KeyCode::Char('q'),
                ..
            }) => {
                return WindowActionResult::Exit;
            }
            _ => (),
        }
        WindowActionResult::Continue
    }
}
