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
            ],
            vec![],
            vec!["Todo List".yellow().bold().underlined()],
            vec![
                "Todo Items".green().bold(),
                " are tasks which must be completed, and they can be categorised into ".into(),
                "Buckets".green().bold(),
                ".".into(),
            ],
            vec![
                "Creating Todo Items and Buckets:".yellow().bold(),
                " Go to the ".into(),
                "Todo List".yellow().bold(),
                " section and type the name of a new ".into(),
                "Todo Item".green().bold(),
                " or ".into(),
                "Bucket".green().bold(),
                ", then press ".into(),
                "<Enter>".blue().bold(),
                " to create the item or bucket.".into(),
            ],
            vec![
                "Moving a Todo Item to a different Bucket:".yellow().bold(),
                " Select a ".into(),
                "Todo Item".green().bold(),
                " and press ".into(),
                "<Space>".blue().bold(),
                " to select it to be moved, then select a ".into(),
                "Bucket".green().bold(),
                " and press ".into(),
                "<Space>".blue().bold(),
                " to move the item to that ".into(),
                "Bucket".green().bold(),
                ".".into()
            ],
            vec![
                "Deleting a Bucket:".yellow().bold(),
                " Press ".into(),
                "<Enter>".blue().bold(),
                " when an empty ".into(),
                "Bucket".green().bold(),
                " is selected to delete it.".into()
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
