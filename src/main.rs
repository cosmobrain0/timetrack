mod help;
mod input_widget;
mod state;
mod todo;
mod track;

use std::collections::HashMap;
use std::path::PathBuf;
use std::str::FromStr;
use std::time::Duration;

use chrono::Utc;
use color_eyre::Result;
use help::HelpWindow;
use ratatui::crossterm::event;
use ratatui::layout::{Constraint, Layout};
use ratatui::style::{Color, Stylize};
use ratatui::text::Line;
use ratatui::widgets::{Block, Paragraph, Widget};
use ratatui::{DefaultTerminal, Frame};
use state::{State, StateBuilder};
use todo::TodoWindow;
use track::TrackWindow;

fn main() -> Result<()> {
    color_eyre::install()?;
    let mut terminal = ratatui::init();
    let result = App::new()?.run(&mut terminal);
    ratatui::restore();
    result
}

trait Window: std::fmt::Debug {
    fn draw(&self, state: &State, frame: &mut Frame, area: ratatui::layout::Rect);
    fn handle_event(&mut self, state: &mut State, event: &event::Event) -> WindowActionResult;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum AppWindow {
    Track,
    Todo,
    Help,
}

struct App {
    state: State,
    exit: bool,
    windows: HashMap<AppWindow, Box<dyn Window>>,
    current_window: AppWindow,
}
impl App {
    fn new() -> Result<Self> {
        let state = load_state()?;
        let mut windows = HashMap::new();
        windows.insert(
            AppWindow::Track,
            Box::new(TrackWindow::new()) as Box<dyn Window>,
        );
        windows.insert(
            AppWindow::Todo,
            Box::new(TodoWindow::new()) as Box<dyn Window>,
        );
        windows.insert(
            AppWindow::Help,
            Box::new(HelpWindow::new()) as Box<dyn Window>,
        );
        Ok(Self {
            state,
            exit: false,
            current_window: AppWindow::Track,
            windows,
        })
    }

    fn run(&mut self, terminal: &mut DefaultTerminal) -> Result<()> {
        while !self.exit {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events()?;
        }
        Ok(())
    }

    fn draw(&self, frame: &mut Frame) {
        let [header_area, main_area] =
            Layout::vertical([Constraint::Length(3), Constraint::Fill(1)]).areas(frame.area());

        frame.render_widget(
            &HeaderWidget {
                tabs: vec!["Track Activities", "Todo List", "Help"],
                selected: match self.current_window {
                    AppWindow::Track => 0,
                    AppWindow::Todo => 1,
                    AppWindow::Help => 2,
                },
            },
            header_area,
        );

        self.windows[&self.current_window].draw(&self.state, frame, main_area);
    }

    fn handle_events(&mut self) -> std::io::Result<()> {
        // NOTE: 10 seconds might be a long time!
        if event::poll(Duration::from_secs(10))? {
            // NOTE: this is NOT blocking!
            let evt = event::read()?;
            let result = self
                .windows
                .get_mut(&self.current_window)
                .unwrap()
                .handle_event(&mut self.state, &evt);
            match result {
                WindowActionResult::Continue => (),
                WindowActionResult::Exit => {
                    if self.state.pomo_minutes().is_none() {
                        self.exit = true;
                    }
                }
                WindowActionResult::FirstWindow => self.current_window = AppWindow::Track,
                WindowActionResult::SecondWindow => self.current_window = AppWindow::Todo,
                WindowActionResult::ThirdWindow => self.current_window = AppWindow::Help,
            }
        }
        if let Some(pomo_minutes) = self.state.pomo_minutes() {
            if self
                .state
                .current_session_duration()
                .expect("should have a current session")
                .num_minutes()
                >= pomo_minutes as i64
            {
                let activity_id = self
                    .state
                    .current_id()
                    .expect("should be able to get current ID");
                self.state
                    .end_activity(true)
                    .expect("should be able to end activity in pomo!");
                let activity_name = self
                    .state
                    .activity_by_id(activity_id)
                    .expect("should be able to get activity")
                    .name();

                #[cfg(feature = "mac-notifications")]
                mac_notification_sys::send_notification(
                    "Pomodoro Session Over!!",
                    Some(activity_name),
                    &format!("You've worked for {pomo_minutes}min on {activity_name}!"),
                    Some(
                        mac_notification_sys::Notification::new()
                            .asynchronous(true)
                            .wait_for_click(true),
                    ),
                )
                .expect("should be able to send notification!");
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum WindowActionResult {
    Continue,
    Exit,
    FirstWindow,
    SecondWindow,
    ThirdWindow,
}

fn load_state() -> Result<State> {
    let path = stored_state_file_path()?;
    let stored_state: StateBuilder = if path.exists() {
        serde_json::from_str(&std::fs::read_to_string(path)?)?
    } else {
        if let Some(dirs) = path.parent() {
            std::fs::create_dir_all(dirs)?;
        }
        std::fs::write(path, "{}")?;
        serde_json::from_str("{}")?
    };
    let state: State = stored_state.into();
    if state.date() == Utc::now().date_naive() {
        Ok(state)
    } else {
        Ok(state.refresh())
    }
}

fn stored_state_file_path() -> Result<PathBuf, color_eyre::eyre::Error> {
    Ok(PathBuf::from_str(
        if let Ok(file_path) = std::env::var("TIMETRACK_STATE_FILE_PATH") {
            file_path
        } else {
            let home = std::env::var("HOME")?;
            format!("{home}/.timetrack/state.json")
        }
        .as_str(),
    )
    .unwrap())
}

fn instruction_line(values: Vec<(&str, &str)>) -> Line<'static> {
    Line::from(
        values
            .into_iter()
            .flat_map(|(action, keybind)| {
                vec![
                    format!(" {action} ").into(),
                    format!("<{keybind}>").blue().bold(),
                ]
            })
            .chain([" ".into()])
            .collect::<Vec<_>>(),
    )
    .centered()
}

struct HeaderWidget<'a> {
    tabs: Vec<&'a str>,
    selected: usize,
}
impl<'a> Widget for &HeaderWidget<'a> {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        Paragraph::new(vec![Line::from(
            self.tabs
                .iter()
                .enumerate()
                .map(|(i, x)| (i + 1, x))
                .map(|(i, x)| (i == self.selected + 1, format!("{x} [{i}] ")))
                .map(|(selected, text)| if selected { text.yellow() } else { text.into() })
                .collect::<Vec<_>>(),
        )])
        .block(Block::bordered().style(Color::DarkGray))
        .left_aligned()
        .render(area, buf);
    }
}
