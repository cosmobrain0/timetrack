use ratatui::crossterm::event::{Event, KeyCode, KeyEvent};
use ratatui::layout::Rect;
use ratatui::style::Stylize;
use ratatui::widgets::{List, Widget};
use ratatui::{
    Frame,
    layout::{Constraint, Layout},
    style::{Color, Style},
    widgets::Block,
};
use tui_input::{Input, backend::crossterm::EventHandler};

use crate::input_widget::InputWidget;
use crate::state::{State, TodoItem};
use crate::{Window, WindowActionResult, instruction_line};

#[derive(Debug)]
pub(crate) struct TodoWindow {
    focused_widget: TodoWidget,
    selected: usize,
    input: Input,
}
impl TodoWindow {
    pub fn new() -> Self {
        Self {
            focused_widget: TodoWidget::Todos,
            selected: 0,
            input: Input::new(String::new()),
        }
    }
}
impl Window for TodoWindow {
    fn draw(&self, state: &State, frame: &mut Frame, area: Rect) {
        let [list_area, input_area] =
            Layout::vertical([Constraint::Min(3), Constraint::Length(3)]).areas(area);

        frame.render_widget(
            &InputWidget {
                is_focused: self.focused_widget == TodoWidget::Input,
                input: &self.input,
                title: "New Todo",
            },
            input_area,
        );

        frame.render_widget(
            &TodoListWidget {
                is_focused: self.focused_widget == TodoWidget::Todos,
                todos: state.get_todos().collect(),
                selected: self.selected,
            },
            list_area,
        );
    }

    fn handle_event(&mut self, state: &mut State, event: &Event) -> WindowActionResult {
        match event {
            Event::Key(KeyEvent {
                code: KeyCode::Tab, ..
            }) => {
                self.focused_widget = match self.focused_widget {
                    TodoWidget::Todos => TodoWidget::Input,
                    TodoWidget::Input => TodoWidget::Buckets,
                    TodoWidget::Buckets => TodoWidget::Todos,
                }
            }
            Event::Key(KeyEvent {
                code: KeyCode::Enter,
                ..
            }) => {
                if self.focused_widget == TodoWidget::Input {
                    // TODO: update this to consider the bucket input!
                    state.push_todo(TodoItem::new(self.input.value().to_string(), None));
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
                if self.focused_widget == TodoWidget::Input {
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
                if self.focused_widget != TodoWidget::Input {
                    return WindowActionResult::FirstWindow;
                }
            }
            Event::Key(KeyEvent {
                code: KeyCode::Char('2'),
                ..
            }) => {
                if self.focused_widget != TodoWidget::Input {
                    return WindowActionResult::SecondWindow;
                }
            }
            Event::Key(KeyEvent {
                code: KeyCode::Char('3'),
                ..
            }) => {
                if self.focused_widget != TodoWidget::Input {
                    return WindowActionResult::ThirdWindow;
                }
            }

            _ => {
                if self.focused_widget == TodoWidget::Input {
                    self.input.handle_event(event);
                }
            }
        }
        WindowActionResult::Continue
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TodoWidget {
    Todos,
    Input,
    Buckets,
}

struct TodoListWidget<'a> {
    is_focused: bool,
    todos: Vec<&'a TodoItem>,
    selected: usize,
}
impl<'a> Widget for &TodoListWidget<'a> {
    fn render(self, area: Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        let list_style = if !self.is_focused {
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
        // TODO: update this with better formatting!
        List::new(
            self.todos
                .iter()
                .map(|x| {
                    format!(
                        "<{bucket}> {item}",
                        bucket = x.bucket().unwrap_or("N/A"),
                        item = x.item()
                    )
                })
                .enumerate()
                .map(|(i, x)| {
                    if self.is_focused && i == self.selected {
                        x.blue().bold()
                    } else {
                        x.into()
                    }
                }),
        )
        .style(list_style)
        .block(if !self.is_focused {
            Block::bordered().title(" Todo Items ")
        } else {
            Block::bordered()
                .title(" Todo Items ")
                .title_bottom(list_instructions.centered())
        })
        .render(area, buf);
    }
}
