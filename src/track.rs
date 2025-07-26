use std::{
    io::Write,
    sync::{Arc, atomic::AtomicBool},
};

use chrono::{Duration, Utc};
use crossterm::{cursor, execute, style};
use ratatui::{
    Frame,
    crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers},
    layout::{Constraint, Layout},
    style::{Color, Style, Stylize},
    symbols::line::THICK_HORIZONTAL_DOWN,
    text::Line,
    widgets::{Block, Borders, List, Paragraph, Widget, Wrap},
};
use tui_input::{Input, backend::crossterm::EventHandler};

use crate::{
    WindowActionResult,
    input_widget::InputWidget,
    instruction_line,
    state::{self, Activity, ActivityId, StartActivityError, State},
};

// pub fn pomodoro(current_state: &mut State, minutes: usize) {
//     let (activity, session_minutes) = match find_recommended_action(current_state) {
//         Ok(activity) => (
//             activity,
//             activity
//                 .target_minutes()
//                 .saturating_sub(activity.acheived_minutes())
//                 .min(minutes),
//         ),
//         Err(FindRecommendedActionError::NoMoreTasks) => {
//             println!("You have no more tasks! You're done!");
//             return;
//         }
//         Err(FindRecommendedActionError::Ongoing(activity)) => {
//             println!(
//                 "Stop your current task before running pomo!\n{}",
//                 current_state.format_activity(activity, None)
//             );
//             return;
//         }
//         Err(FindRecommendedActionError::OngoingCompleted(activity)) => {
//             println!(
//                 "Stop your current task!\n{}",
//                 current_state.format_activity(activity, None)
//             );
//             return;
//         }
//     };

//     let interrupted = Arc::new(AtomicBool::new(false));

//     let interrupted_clone = Arc::clone(&interrupted);
//     ctrlc::set_handler(move || {
//         interrupted_clone.store(true, std::sync::atomic::Ordering::Relaxed);
//     })
//     .expect("should be able to set ctrlc handler!");

//     println!("Work on this task for {session_minutes}min!");
//     let activity_id = activity.id();
//     let duration = Duration::seconds(session_minutes as i64 * 60);
//     current_state
//         .start_activity_pomo(activity_id, Some(session_minutes))
//         .expect("should be able to start activity");
//     let start = Utc::now();
//     let end = start + duration;
//     let activity = current_state
//         .activity_by_id(activity_id)
//         .expect("should be able to get ID of started activity");
//     println!("{}", current_state.format_activity(activity, None));

//     const TIMER_LENGTH: usize = 10;
//     let mut stdout = std::io::stdout();
//     while Utc::now() < end && !interrupted.load(std::sync::atomic::Ordering::Relaxed) {
//         let length = (TIMER_LENGTH as f64 * (Utc::now() - start).num_seconds() as f64
//             / duration.num_seconds() as f64)
//             .floor() as usize;
//         execute!(
//             stdout,
//             cursor::MoveToColumn(0),
//             style::Print(format!(
//                 "[{bars}>{padding}]",
//                 bars = "=".repeat(length),
//                 padding = " ".repeat(TIMER_LENGTH.saturating_sub(length))
//             ))
//         )
//         .expect("should be able to draw progress bar");
//         stdout.flush().unwrap();
//         std::thread::sleep(std::time::Duration::from_millis(50));
//     }
//     println!("\n");

//     if interrupted.load(std::sync::atomic::Ordering::Relaxed) {
//         let time_left = ((end - Utc::now()).num_seconds() as f64 / 60.0).ceil() as usize;
//         println!("You've ended the session {time_left}min early!");
//     } else {
//         println!("Stop working!");
//     }
//     current_state
//         .end_activity(true)
//         .expect("should be able to end activity in pomo!");

//     let activity = current_state
//         .activity_by_id(activity_id)
//         .expect("should be able to get ID of finished activity");
//     let activity_name = activity.name();
//     println!("{}", current_state.format_activity(activity, None));

//     mac_notification_sys::send_notification(
//         "Pomodoro Session Over!!",
//         Some(activity_name),
//         &format!("You've worked for {session_minutes}min on {activity_name}!"),
//         Some(
//             mac_notification_sys::Notification::new()
//                 .asynchronous(true)
//                 .wait_for_click(true),
//         ),
//     )
//     .expect("should be able to send notification!");
// }

// pub fn overwrite_time(current_state: &mut State, id: usize, minutes: usize) {
//     if let Some(id) = current_state.get_by_raw_id(id) {
//         current_state.overwrite_time(id, minutes).unwrap();
//         println!("Set acheived minutes of {id} to {minutes}min!");
//     } else {
//         println!("There is no activity with that id!");
//     }
// }

// pub fn register_time(current_state: &mut State, id: usize, minutes: usize) {
//     if let Some(id) = current_state.get_by_raw_id(id) {
//         current_state.add_time(id, minutes).unwrap();
//         println!("Added {minutes}min to {id}!");
//     } else {
//         println!("There is no activity with that id!");
//     }
// }

// pub fn end_activity(current_state: &mut State) {
//     match current_state.end_activity(false) {
//         Ok(()) => println!("Ended activity!"),
//         Err(state::EndActivityError::NoCurrentActivity) => {
//             println!("There is no ongoing activity!")
//         }
//         Err(state::EndActivityError::PomoOngoing) => {
//             println!("You must cancel the ongoing pomo session first!");
//         }
//     }
// }

// pub fn start_activity(current_state: &mut State, id: usize) {
//     if let Some(id) = current_state.get_by_raw_id(id) {
//         match current_state.start_activity(id) {
//             Ok(()) => println!("Started activity {id}!"),
//             Err(StartActivityError::AlreadyOngoing) => {
//                 println!("There is already an ongoing activity!")
//             }
//             Err(state::StartActivityError::InvalidId) => {
//                 println!("There is no activity with that ID!")
//             } // this should be unreachable but whatever
//         }
//     } else {
//         println!("There is no activity with that ID!");
//     }
// }

// pub fn del_activity(current_state: &mut State, id: usize) {
//     if let Some(id) = current_state.get_by_raw_id(id) {
//         match current_state.delete(id) {
//             Ok(()) => println!("Deleted activity {id}!"),
//             Err(state::DeletionError::PomoOngoing) => {
//                 println!("You must end the currently ongoing pomo session first!")
//             }
//             Err(state::DeletionError::InvalidId) => unreachable!(),
//         }
//     } else {
//         println!("There is no activity with id {id}!");
//     }
// }

// pub fn list_activities(current_state: &State) {
//     let current_activity = current_state.current_id();
//     let current_duration = current_state.current_session_duration();
//     let mut activities = current_state.activities();
//     activities.sort_by_key(|x| {
//         x.acheived_minutes()
//             + current_activity
//                 .is_some_and(|id| id == x.id())
//                 .then(|| current_duration.unwrap().num_minutes().max(0) as usize)
//                 .unwrap_or(0)
//     });
//     let activities = activities;
//     let max_name_length = activities
//         .iter()
//         .map(|x| x.name().chars().count() + 1)
//         .max()
//         .unwrap_or(1);
//     if activities.is_empty() {
//         println!("No activities!");
//     }
//     for activity in activities {
//         println!(
//             "{}",
//             current_state.format_activity(activity, Some(max_name_length))
//         );
//     }
// }

// pub enum FindRecommendedActionError<'a> {
//     NoMoreTasks,
//     Ongoing(&'a Activity),
//     OngoingCompleted(&'a Activity),
// }

// pub fn find_recommended_action(
//     current_state: &State,
// ) -> Result<&Activity, FindRecommendedActionError<'_>> {
//     if let Some(current_task) = current_state.current_activity() {
//         if current_task.acheived_minutes()
//             + current_state
//                 .current_session_duration()
//                 .unwrap()
//                 .num_minutes()
//                 .max(0) as usize
//             >= current_task.target_minutes()
//         {
//             Err(FindRecommendedActionError::OngoingCompleted(current_task))
//         } else {
//             Err(FindRecommendedActionError::Ongoing(current_task))
//         }
//     } else if let Some(activity) = current_state
//         .activities()
//         .iter()
//         .filter(|x| x.target_minutes() > x.acheived_minutes())
//         .reduce(|a, b| {
//             if a.acheived_minutes() < b.acheived_minutes() {
//                 a
//             } else {
//                 b
//             }
//         })
//     {
//         Ok(activity)
//     } else {
//         Err(FindRecommendedActionError::NoMoreTasks)
//     }
// }

// pub fn list_recommended_action(current_state: &State) {
//     match find_recommended_action(current_state) {
//         Ok(activity) => println!("{}", current_state.format_activity(activity, None)),
//         Err(FindRecommendedActionError::NoMoreTasks) => {
//             println!("There are no more tasks! You're done!")
//         }
//         Err(FindRecommendedActionError::OngoingCompleted(activity)) => {
//             let task_formatted = current_state.format_activity(activity, None);
//             println!("You're currently doing:\n{task_formatted}\nStop the current task!");
//         }
//         Err(FindRecommendedActionError::Ongoing(activity)) => {
//             let task_formatted = current_state.format_activity(activity, None);
//             println!("Continue with your current task!\n{task_formatted}");
//         }
//     }
// }

// pub fn add_activity(current_state: &mut State, name: String, target_minutes: usize) {
//     let new_activity_id = current_state.add_activity(name.clone(), target_minutes);
//     println!("Added activity {new_activity_id}: {name} with target {target_minutes}min");
// }

// pub fn change_target_time(current_state: &mut State, id: usize, target_minutes: usize) {
//     if let Some(activity) = current_state.get_by_raw_id_mut(id) {
//         activity.set_target_minutes(target_minutes);
//         println!("Overwritten target minutes!");
//     } else {
//         println!("That activity ID is invalid!");
//     }
// }

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
    OverwriteActivity(ActivityId),
    RegisterActivity(ActivityId),
    ChangeTarget(ActivityId),
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

    pub fn draw(&self, state: &State, frame: &mut Frame) {
        let [
            activities_area,
            text_input_area,
            timer_input_area,
            ongoing_area,
        ] = {
            let [upper_area, lower_area] =
                Layout::vertical([Constraint::Min(3), Constraint::Length(3)]).areas(frame.area());
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
                ongoing: state.current_activity().map(Activity::clone),
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
                TimerInputPurpose::OverwriteActivity(activity_id) => {
                    let _ = state.overwrite_time(activity_id, self.timer_input);
                    self.focused_widget = Activities;
                    self.timer_input_purpose = TimerInputPurpose::NewActivity;
                }
                TimerInputPurpose::RegisterActivity(activity_id) => {
                    let _ = state.add_time(activity_id, self.timer_input);
                    self.focused_widget = Activities;
                    self.timer_input_purpose = TimerInputPurpose::NewActivity;
                }
                TimerInputPurpose::ChangeTarget(activity_id) => {
                    if let Some(activity) = state.get_by_id_mut(activity_id) {
                        activity.set_target_minutes(self.timer_input);
                    }
                    self.focused_widget = Activities;
                    self.timer_input_purpose = TimerInputPurpose::NewActivity;
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
                code: KeyCode::Char('r'),
                ..
            }) if self.focused_widget == Activities => {
                if let Some(activity_id) = self.selected_activity_id(state) {
                    self.timer_input_purpose = TimerInputPurpose::RegisterActivity(activity_id);
                    self.focused_widget = TimerInput;
                }
            }
            Event::Key(KeyEvent {
                code: KeyCode::Char('o'),
                ..
            }) if self.focused_widget == Activities => {
                if let Some(activity_id) = self.selected_activity_id(state) {
                    self.timer_input_purpose = TimerInputPurpose::OverwriteActivity(activity_id);
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
                        x.into()
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
                        r = remaining_time.to_string(),
                        a = acheived_time.to_string(),
                        t = (acheived_time + remaining_time).to_string()
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
                .borders(Borders::all()),
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
                        TimerInputPurpose::OverwriteActivity(x) => {
                            format!(
                                " Overwrite Time for {}",
                                self.selected_activity_name.unwrap_or_default()
                            )
                        }
                        TimerInputPurpose::RegisterActivity(x) => format!(
                            " Register Time for {}",
                            self.selected_activity_name.unwrap_or_default()
                        ),
                        TimerInputPurpose::ChangeTarget(x) => format!(
                            " Change Target for {} ",
                            self.selected_activity_name.unwrap_or_default()
                        ),
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
