use ratatui::{
    crossterm::event::{Event, KeyCode, KeyEvent},
    style::Stylize,
    text::{Line, Span},
    widgets::{Block, Paragraph, Wrap},
};

use crate::{Window, WindowActionResult};

#[derive(Debug)]
pub struct HelpWindow {
    data: Vec<Vec<Span<'static>>>,
}
impl HelpWindow {
    pub fn new() -> Self {
        let data = vec![
            vec!["Navigation".yellow().bold().underlined()],
            vec![
                "Change Tabs:".yellow().bold(),
                " Press the corresponding number on the keyboard (tabs shown at the top).".into(),
            ],
            vec![
                "Switch between widgets:".yellow().bold(),
                " Press ".into(),
                "<Tab>".blue().bold(),
                " to switch between widgets.".into(),
            ],
            vec![
                "Quit:".yellow().bold(),
                " Press ".into(),
                "<q>".blue().bold(),
                " when an input widget is not selected, and a pomodoro session is not ongoing."
                    .into(),
            ],
            vec![],
            vec!["Activities".yellow().bold().underlined()],
            vec![
                "Activities".green().bold(),
                " are tasks with a daily target (the amount of time you should spend on them every day).".into(),
            ],
            vec![
                "Create an Activity:".yellow().bold(),
                " Go to the ".into(),
                "Activities".green().bold(),
                " tab, input the name of the activity, then input the daily target and press Enter.".into(),
            ],
            vec![
                "Start a Pomodoro Session:".yellow().bold(),
                " Press ".into(),
                "<p>".blue().bold(),
                " on the ".into(),
                "Activities".green().bold(),
                " section, then modify the duration of the session if required, then press ".into(),
                "<Enter>".blue().bold(),
                " to start the session.".into(),
            ],
            vec![
                "Ending a Pomodoro Session:".yellow().bold(),
                " The session will end automatically when the timer is up. If the ".into(),
                "notifications".green().bold(),
                " feature is activated, then you will get a notification when the session is over. The session can be stopped early by going to the ".into(),
                "Ongoing".green().bold(), 
                " widget and pressing ".into(),
                "<Backspace>".blue().bold(),
                ".".into(),
            ]
        ];
        Self { data }
    }
}
impl Window for HelpWindow {
    fn draw(
        &self,
        _state: &crate::state::State,
        frame: &mut ratatui::Frame<'_>,
        main_area: ratatui::prelude::Rect,
    ) {
        frame.render_widget(
            Paragraph::new(
                self.data
                    .iter()
                    .map(|d| Line::from(d.clone()))
                    .collect::<Vec<_>>(),
            )
            .wrap(Wrap { trim: true })
            .centered()
            .block(Block::bordered().title(" Help ")),
            main_area,
        );
    }

    fn handle_event(
        &mut self,
        _state: &mut crate::state::State,
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
