use ratatui::crossterm::event::{Event, KeyCode, KeyEvent};
use ratatui::layout::Rect;
use ratatui::style::Stylize;
use ratatui::widgets::List;
use ratatui::{
    Frame,
    layout::{Constraint, Layout},
    style::{Color, Style},
    widgets::Block,
};
use tui_input::{Input, backend::crossterm::EventHandler};

use crate::input_widget::InputWidget;
use crate::state::State;
use crate::{WindowActionResult, instruction_line};

pub(crate) struct TodoWindow {
    input_focused: bool,
    selected: usize,
    input: Input,
}
impl TodoWindow {
    pub fn new() -> Self {
        Self {
            input_focused: false,
            selected: 0,
            input: Input::new(String::new()),
        }
    }

    pub fn draw(&self, state: &State, frame: &mut Frame, area: Rect) {
        let [list_area, input_area] =
            Layout::vertical([Constraint::Min(3), Constraint::Length(3)]).areas(area);

        frame.render_widget(
            &InputWidget {
                is_focused: self.input_focused,
                input: &self.input,
                title: "New Todo",
            },
            input_area,
        );

        let list_style = if self.input_focused {
            Style::default()
        } else {
            Color::Yellow.into()
        };
        let list_instructions = instruction_line(vec![
            ("Scroll Up", "Up"),
            ("Scroll Down", "Down"),
            ("Delete", "Enter"),
            ("Move Up", "Left"),
            ("Move Down", "Right"),
        ]);
        let list = List::new(state.get_todos().enumerate().map(|(i, x)| {
            if !self.input_focused && i == self.selected {
                x.to_string().blue().bold()
            } else {
                x.into()
            }
        }))
        .style(list_style)
        .block(if self.input_focused {
            Block::bordered().title(" Todo Items ")
        } else {
            Block::bordered()
                .title(" Todo Items ")
                .title_bottom(list_instructions.centered())
        });
        frame.render_widget(list, list_area);
    }

    pub fn handle_event(&mut self, state: &mut State, event: &Event) -> WindowActionResult {
        match event {
            Event::Key(KeyEvent {
                code: KeyCode::Tab, ..
            }) => {
                self.input_focused = !self.input_focused;
            }
            Event::Key(KeyEvent {
                code: KeyCode::Enter,
                ..
            }) => {
                if self.input_focused {
                    state.push_todo(self.input.value().to_string());
                    self.input.reset();
                } else if self.selected < state.todo_count() {
                    let _ = state.delete_todo(self.selected);
                    self.selected = self.selected.min(state.todo_count().saturating_sub(1));
                }
            }
            Event::Key(KeyEvent {
                code: KeyCode::Char('q'),
                ..
            }) => {
                if self.input_focused {
                    self.input.handle_event(event);
                } else {
                    return WindowActionResult::Exit;
                }
            }
            Event::Key(KeyEvent {
                code: KeyCode::Down,
                ..
            }) => {
                self.selected = (self.selected + 1).min(state.todo_count().saturating_sub(1));
            }
            Event::Key(KeyEvent {
                code: KeyCode::Up, ..
            }) => {
                self.selected = self.selected.saturating_sub(1);
            }
            Event::Key(KeyEvent {
                code: KeyCode::Left,
                ..
            }) => {
                if self.selected > 0 && self.selected < state.todo_count() {
                    let _ = state.swap_todos(self.selected, self.selected - 1);
                    self.selected -= 1;
                }
            }
            Event::Key(KeyEvent {
                code: KeyCode::Right,
                ..
            }) => {
                if state.todo_count() > 1 && self.selected < state.todo_count() - 1 {
                    let _ = state.swap_todos(self.selected, self.selected + 1);
                    self.selected += 1;
                }
            }
            Event::Key(KeyEvent {
                code: KeyCode::Char('1'),
                ..
            }) => {
                if !self.input_focused {
                    return WindowActionResult::FirstWindow;
                }
            }
            Event::Key(KeyEvent {
                code: KeyCode::Char('2'),
                ..
            }) => {
                if !self.input_focused {
                    return WindowActionResult::SecondWindow;
                }
            }

            _ => {
                if self.input_focused {
                    self.input.handle_event(event);
                }
            }
        }
        WindowActionResult::Continue
    }
}
