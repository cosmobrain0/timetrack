use ratatui::{
    Frame,
    crossterm::event::{Event, KeyCode, KeyEvent},
    layout::{Constraint, Layout, Rect},
    style::{Color, Style, Stylize},
    text::Line,
    widgets::{Block, Borders, List, Paragraph, Widget, Wrap},
};
use tui_input::{Input, backend::crossterm::EventHandler};

use crate::{
    WindowActionResult,
    input_widget::InputWidget,
    instruction_line,
    state::{Activity, ActivityId, State},
};

pub enum FindRecommendedActionError {
    NoMoreTasks,
    Ongoing,
    OngoingCompleted,
}

pub fn find_recommended_action(
    current_state: &State,
) -> Result<&Activity, FindRecommendedActionError> {
    if let Some(current_task) = current_state.current_activity() {
        if current_task.acheived_minutes()
            + current_state
                .current_session_duration()
                .unwrap()
                .num_minutes()
                .max(0) as usize
            >= current_task.target_minutes()
        {
            Err(FindRecommendedActionError::OngoingCompleted)
        } else {
            Err(FindRecommendedActionError::Ongoing)
        }
    } else if let Some(activity) = current_state
        .activities()
        .filter(|x| x.target_minutes() > x.acheived_minutes())
        .reduce(|a, b| {
            if a.acheived_minutes() < b.acheived_minutes() {
                a
            } else {
                b
            }
        })
    {
        Ok(activity)
    } else {
        Err(FindRecommendedActionError::NoMoreTasks)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TrackWindowWidget {
    Activities,
    TextInput,
    TimerInput,
    Ongoing,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TimerInputPurpose {
    NewActivity,
    OverwriteActivity,
    RegisterActivity,
    ChangeTarget,
    StartPomodoro,
}

pub struct TrackWindow {
    focused_widget: TrackWindowWidget,
    text_input: Input,
    timer_input: usize,
    timer_input_purpose: TimerInputPurpose,
    selected_activity: usize,
}
impl TrackWindow {
    pub fn new() -> Self {
        Self {
            focused_widget: TrackWindowWidget::Activities,
            text_input: Input::new(String::new()),
            timer_input_purpose: TimerInputPurpose::NewActivity,
            timer_input: 60,
            selected_activity: 0,
        }
    }

    pub fn draw(&self, state: &State, frame: &mut Frame, area: Rect) {
        let [
            activities_area,
            text_input_area,
            timer_input_area,
            ongoing_area,
        ] = {
            let [upper_area, lower_area] =
                Layout::vertical([Constraint::Min(3), Constraint::Length(3)]).areas(area);
            let [activities_area, ongoing_area] =
                Layout::horizontal([Constraint::Percentage(70), Constraint::Percentage(30)])
                    .areas(upper_area);
            let [text_input_area, timer_input_area] =
                Layout::horizontal([Constraint::Percentage(70), Constraint::Min(6)])
                    .areas(lower_area);
            [
                activities_area,
                text_input_area,
                timer_input_area,
                ongoing_area,
            ]
        };

        frame.render_widget(
            &ActivitiesWidget {
                state,
                is_focused: self.focused_widget == TrackWindowWidget::Activities,
                selected_activity: self.selected_activity,
            },
            activities_area,
        );
        frame.render_widget(
            &OngoingWidget {
                ongoing: state.current_activity().cloned(),
                pomodoro: state.pomo_minutes().map(|total_minutes| {
                    let acheived_time = state
                        .current_session_duration()
                        .unwrap()
                        .num_minutes()
                        .max(0) as usize;
                    PomodoroInfo {
                        acheived_time,
                        remaining_time: total_minutes.saturating_sub(acheived_time),
                    }
                }),
                is_focused: self.focused_widget == TrackWindowWidget::Ongoing,
                state,
            },
            ongoing_area,
        );
        frame.render_widget(
            &InputWidget {
                is_focused: self.focused_widget == TrackWindowWidget::TextInput,
                input: &self.text_input,
                title: "Add Activity",
            },
            text_input_area,
        );
        frame.render_widget(
            &TimerInputWidget {
                value: self.timer_input,
                is_focused: self.focused_widget == TrackWindowWidget::TimerInput,
                purpose: self.timer_input_purpose,
                selected_activity_name: self.selected_activity_name(state),
            },
            timer_input_area,
        );
    }

    pub fn handle_event(&mut self, state: &mut State, event: &Event) -> WindowActionResult {
        use TrackWindowWidget::*;
        match event {
            Event::Key(KeyEvent {
                code: KeyCode::Tab, ..
            }) => {
                self.focused_widget = match self.focused_widget {
                    Activities => TextInput,
                    TextInput => TimerInput,
                    TimerInput => Ongoing,
                    Ongoing => Activities,
                }
            }
            Event::Key(KeyEvent {
                code: KeyCode::Enter,
                ..
            }) if self.focused_widget == TextInput => {
                if !self.text_input.value().is_empty() {
                    self.timer_input_purpose = TimerInputPurpose::NewActivity;
                    self.focused_widget = TimerInput;
                }
            }
            Event::Key(KeyEvent {
                code: KeyCode::Enter,
                ..
            }) if self.focused_widget == TimerInput => match self.timer_input_purpose {
                TimerInputPurpose::NewActivity => {
                    if !self.text_input.value().is_empty() {
                        state.add_activity(self.text_input.value().to_string(), self.timer_input);
                        self.text_input.reset();
                        self.focused_widget = Activities;
                    } else {
                        self.focused_widget = TextInput;
                    }
                }
                TimerInputPurpose::OverwriteActivity => {
                    if let Some(activity_id) = self.selected_activity_id(state) {
                        let _ = state.overwrite_time(activity_id, self.timer_input);
                        self.focused_widget = Activities;
                        self.timer_input_purpose = TimerInputPurpose::NewActivity;
                    }
                }
                TimerInputPurpose::RegisterActivity => {
                    if let Some(activity_id) = self.selected_activity_id(state) {
                        let _ = state.add_time(activity_id, self.timer_input);
                        self.focused_widget = Activities;
                        self.timer_input_purpose = TimerInputPurpose::NewActivity;
                    }
                }
                TimerInputPurpose::ChangeTarget => {
                    if let Some(activity_id) = self.selected_activity_id(state) {
                        if let Some(activity) = state.get_by_id_mut(activity_id) {
                            activity.set_target_minutes(self.timer_input);
                        }
                    }
                    self.focused_widget = Activities;
                    self.timer_input_purpose = TimerInputPurpose::NewActivity;
                }
                TimerInputPurpose::StartPomodoro => {
                    if let Ok(activity) = find_recommended_action(state) {
                        let id = activity.id();
                        let _ = state.start_activity_pomo(id, Some(self.timer_input));
                    }
                }
            },
            Event::Key(KeyEvent {
                code: KeyCode::Char('q'),
                ..
            }) if self.focused_widget != TextInput => {
                return WindowActionResult::Exit;
            }
            Event::Key(KeyEvent {
                code: KeyCode::Down,
                ..
            }) if self.focused_widget == Activities => {
                self.selected_activity =
                    (self.selected_activity + 1).min(state.activities_count().saturating_sub(1));
            }
            Event::Key(KeyEvent {
                code: KeyCode::Down,
                ..
            }) if self.focused_widget == TimerInput => {
                self.timer_input = self.timer_input.saturating_sub(1);
            }
            Event::Key(KeyEvent {
                code: KeyCode::Up, ..
            }) if self.focused_widget == Activities => {
                self.selected_activity = self.selected_activity.saturating_sub(1);
            }
            Event::Key(KeyEvent {
                code: KeyCode::Up, ..
            }) if self.focused_widget == TimerInput => {
                self.timer_input = self.timer_input.saturating_add(1);
            }
            Event::Key(KeyEvent {
                code: KeyCode::Char(' '),
                ..
            }) if self.focused_widget == Activities => {
                if let Some(id) = self.selected_activity_id(state) {
                    if state.current_id().is_some_and(|x| x == id) {
                        let _ = state.end_activity(false);
                    } else {
                        let _ = state.start_activity(id);
                    }
                }
            }
            Event::Key(KeyEvent {
                code: KeyCode::Backspace,
                ..
            }) if self.focused_widget == Activities => {
                if let Some(id) = state
                    .activities()
                    .nth(self.selected_activity)
                    .map(Activity::id)
                {
                    let _ = state.delete(id);
                    self.selected_activity = self
                        .selected_activity
                        .min(state.activities_count().saturating_sub(1));
                }
            }
            Event::Key(KeyEvent {
                code: KeyCode::Backspace,
                ..
            }) if self.focused_widget == TimerInput => {
                self.timer_input = 0;
            }
            Event::Key(KeyEvent {
                code: KeyCode::Backspace,
                ..
            }) if self.focused_widget == Ongoing => {
                if state.pomo_minutes().is_some() {
                    let _ = state.end_activity(true);
                }
            }
            Event::Key(KeyEvent {
                code: KeyCode::Char('1'),
                ..
            }) if self.focused_widget != TextInput => {
                return WindowActionResult::FirstWindow;
            }
            Event::Key(KeyEvent {
                code: KeyCode::Char('2'),
                ..
            }) if self.focused_widget != TextInput => {
                return WindowActionResult::SecondWindow;
            }
            Event::Key(KeyEvent {
                code: KeyCode::Char('3'),
                ..
            }) if self.focused_widget != TextInput => {
                return WindowActionResult::ThirdWindow;
            }
            Event::Key(KeyEvent {
                code: KeyCode::Char('r'),
                ..
            }) if self.focused_widget == Activities => {
                if self.selected_activity_id(state).is_some() {
                    self.timer_input_purpose = TimerInputPurpose::RegisterActivity;
                    self.focused_widget = TimerInput;
                }
            }
            Event::Key(KeyEvent {
                code: KeyCode::Char('o'),
                ..
            }) if self.focused_widget == Activities => {
                if self.selected_activity_id(state).is_some() {
                    self.timer_input_purpose = TimerInputPurpose::OverwriteActivity;
                    self.focused_widget = TimerInput;
                }
            }
            Event::Key(KeyEvent {
                code: KeyCode::Char('p'),
                ..
            }) if self.focused_widget == Activities || self.focused_widget == Ongoing => {
                if let Ok(ideal_session_minutes) = find_recommended_action(state).map(|activity| {
                    activity
                        .target_minutes()
                        .saturating_sub(activity.acheived_minutes())
                        .min(30)
                }) {
                    self.focused_widget = TimerInput;
                    self.timer_input = ideal_session_minutes;
                    self.timer_input_purpose = TimerInputPurpose::StartPomodoro;
                }
            }
            Event::Key(KeyEvent {
                code: KeyCode::Char('c'),
                ..
            }) if self.focused_widget == Activities => {
                if self.selected_activity_id(state).is_some() {
                    self.timer_input_purpose = TimerInputPurpose::ChangeTarget;
                    self.focused_widget = TimerInput;
                }
            }
            _ => {
                if self.focused_widget == TextInput {
                    self.text_input.handle_event(event);
                }
            }
        }

        WindowActionResult::Continue
    }

    fn selected_activity_id(&self, state: &State) -> Option<ActivityId> {
        state
            .activities()
            .nth(self.selected_activity)
            .map(Activity::id)
    }

    fn selected_activity_name<'a>(&self, state: &'a State) -> Option<&'a str> {
        state
            .activities()
            .nth(self.selected_activity)
            .map(Activity::name)
    }
}

struct ActivitiesWidget<'a> {
    state: &'a State,
    is_focused: bool,
    selected_activity: usize,
}
impl<'a> Widget for &ActivitiesWidget<'a> {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        let activities_instructions = instruction_line(vec![
            ("Scroll Up", "Up"),
            ("Scroll Down", "Down"),
            ("Start", "Space"),
            ("Delete", "Backspace"),
            ("Register Time", "R"),
            ("Overwrite Time", "O"),
            ("Start Pomodoro", "P"),
            ("Change Target", "C"),
        ]);
        let max_name_length: usize = self
            .state
            .activities()
            .map(|x| x.name().chars().count())
            .max()
            .unwrap_or(0)
            + 1;
        List::new(
            self.state
                .activities()
                .map(|x| self.state.format_activity(x, Some(max_name_length)))
                .enumerate()
                .map(|(i, x)| {
                    if i == self.selected_activity && self.is_focused {
                        x.blue().bold()
                    } else {
                        x
                    }
                }),
        )
        .style(if self.is_focused {
            Color::Yellow.into()
        } else {
            Style::default()
        })
        .block(if self.is_focused {
            Block::bordered()
                .title(" Activities ")
                .title_bottom(activities_instructions.centered())
        } else {
            Block::bordered().title(" Activities ")
        })
        .render(area, buf);
    }
}

struct OngoingWidget<'a> {
    ongoing: Option<Activity>,
    pomodoro: Option<PomodoroInfo>,
    is_focused: bool,
    state: &'a State,
}
impl<'a> Widget for &OngoingWidget<'a> {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        let block_style = if self.is_focused {
            Color::Yellow.into()
        } else {
            Style::default()
        };
        if let Some(ongoing) = &self.ongoing {
            if let Some(PomodoroInfo {
                acheived_time,
                remaining_time,
            }) = self.pomodoro
            {
                Paragraph::new(vec![
                    self.state.format_activity(ongoing, None),
                    Line::from(format!(
                        "Work for {r}min! Acheived {a} / {t} min",
                        r = remaining_time,
                        a = acheived_time,
                        t = (acheived_time + remaining_time)
                    )),
                ])
                .wrap(Wrap { trim: true })
            } else {
                Paragraph::new(self.state.format_activity(ongoing, None))
            }
        } else {
            Paragraph::new("No ongoing session".dark_gray().italic())
        }
        .block(
            Block::new()
                .title(" Ongoing ")
                .style(block_style)
                .borders(Borders::all())
                .title_bottom(if self.is_focused {
                    if self.ongoing.is_some() {
                        instruction_line(vec![("End Pomodoro Session", "Backspace")])
                    } else {
                        instruction_line(vec![("Start Pomodoro Session", "P")])
                    }
                } else {
                    Line::from(vec![])
                }),
        )
        .render(area, buf);
    }
}
struct PomodoroInfo {
    acheived_time: usize,
    remaining_time: usize,
}

struct TimerInputWidget<'a> {
    value: usize,
    is_focused: bool,
    purpose: TimerInputPurpose,
    selected_activity_name: Option<&'a str>,
}
impl<'a> Widget for &TimerInputWidget<'a> {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        Paragraph::new(format!("{value}min", value = self.value))
            .centered()
            .style(if self.is_focused {
                Color::Yellow.into()
            } else {
                Style::default()
            })
            .block(
                Block::bordered()
                    .title(match self.purpose {
                        TimerInputPurpose::NewActivity => " New Activity Target ".to_string(),
                        TimerInputPurpose::OverwriteActivity => {
                            format!(
                                " Overwrite Time for {}",
                                self.selected_activity_name.unwrap_or_default()
                            )
                        }
                        TimerInputPurpose::RegisterActivity => format!(
                            " Register Time for {}",
                            self.selected_activity_name.unwrap_or_default()
                        ),
                        TimerInputPurpose::ChangeTarget => format!(
                            " Change Target for {} ",
                            self.selected_activity_name.unwrap_or_default()
                        ),
                        TimerInputPurpose::StartPomodoro => " Pomodoro Session Length ".to_string(),
                    })
                    .title_bottom(if self.is_focused {
                        instruction_line(vec![
                            ("+1", "Up"),
                            ("-1", "Down"),
                            ("Reset", "Backspace"),
                            ("Confirm", "Enter"),
                        ])
                    } else {
                        Line::from(vec![])
                    }),
            )
            .render(area, buf);
    }
}
