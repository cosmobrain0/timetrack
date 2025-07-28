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
    selected_todo: usize,
    selected_bucket: usize,
    todo_input: Input,
    bucket_input: Input,
}
impl TodoWindow {
    pub fn new() -> Self {
        Self {
            focused_widget: TodoWidget::Todos,
            selected_todo: 0,
            todo_input: Input::new(String::new()),
            bucket_input: Input::new(String::new()),
            selected_bucket: 0,
        }
    }
}
impl Window for TodoWindow {
    fn draw(&self, state: &State, frame: &mut Frame, area: Rect) {
        let [upper_area, input_area] =
            Layout::vertical([Constraint::Min(3), Constraint::Length(3)]).areas(area);
        let [list_area, buckets_area] =
            Layout::horizontal([Constraint::Fill(3), Constraint::Fill(1)]).areas(upper_area);
        let [todo_input_area, bucket_input_area] =
            Layout::horizontal([Constraint::Fill(3), Constraint::Fill(1)]).areas(input_area);

        frame.render_widget(
            &InputWidget {
                is_focused: self.focused_widget == TodoWidget::TodoInput,
                input: &self.todo_input,
                title: "New Todo",
            },
            todo_input_area,
        );

        frame.render_widget(
            &InputWidget {
                is_focused: self.focused_widget == TodoWidget::BucketInput,
                input: &self.bucket_input,
                title: "New Bucket",
            },
            bucket_input_area,
        );

        frame.render_widget(
            &TodoListWidget {
                is_focused: self.focused_widget == TodoWidget::Todos,
                todos: state.get_todos().collect(),
                selected: self.selected_todo,
                selected_bucket: if self.selected_bucket == 0 {
                    None
                } else {
                    state.buckets().get(self.selected_bucket - 1).map(|x| *x)
                },
            },
            list_area,
        );

        frame.render_widget(
            &BucketListWidget {
                is_focused: self.focused_widget == TodoWidget::Buckets,
                buckets: [None]
                    .into_iter()
                    .chain(state.buckets().into_iter().map(Option::Some))
                    .collect(),
                selected: self.selected_bucket,
            },
            buckets_area,
        );
    }

    fn handle_event(&mut self, state: &mut State, event: &Event) -> WindowActionResult {
        match event {
            Event::Key(KeyEvent {
                code: KeyCode::Tab, ..
            }) => {
                self.focused_widget = match self.focused_widget {
                    TodoWidget::Todos => TodoWidget::TodoInput,
                    TodoWidget::TodoInput => TodoWidget::Buckets,
                    TodoWidget::Buckets => TodoWidget::BucketInput,
                    TodoWidget::BucketInput => TodoWidget::Todos,
                }
            }
            Event::Key(KeyEvent {
                code: KeyCode::Enter,
                ..
            }) => {
                if self.focused_widget == TodoWidget::TodoInput {
                    // TODO: update this to consider the bucket input!
                    let bucket = if self.selected_bucket == 0 {
                        None
                    } else {
                        state
                            .buckets()
                            .get(self.selected_bucket - 1)
                            .map(ToString::to_string)
                    };
                    state.push_todo(TodoItem::new(self.todo_input.value().to_string(), bucket));
                    self.todo_input.reset();
                } else if self.focused_widget == TodoWidget::BucketInput {
                    state.create_bucket(self.bucket_input.value().to_string());
                    self.bucket_input.reset();
                } else if self.focused_widget == TodoWidget::Todos {
                    if self.selected_todo < state.todo_count() {
                        let _ = state.delete_todo(self.selected_todo);
                        self.selected_todo =
                            self.selected_todo.min(state.todo_count().saturating_sub(1));
                    }
                } else if self.focused_widget == TodoWidget::Buckets {
                    if self.selected_bucket > 0
                        && let Some(bucket) = state
                            .buckets()
                            .get(self.selected_bucket - 1)
                            .map(|x| x.to_string())
                    {
                        state.delete_bucket(bucket.as_str());
                    }
                }
            }
            Event::Key(KeyEvent {
                code: KeyCode::Char('q'),
                ..
            }) if self.focused_widget != TodoWidget::TodoInput
                && self.focused_widget != TodoWidget::BucketInput =>
            {
                return WindowActionResult::Exit;
            }
            Event::Key(KeyEvent {
                code: KeyCode::Down,
                ..
            }) if self.focused_widget == TodoWidget::Todos => {
                self.selected_todo =
                    (self.selected_todo + 1).min(state.todo_count().saturating_sub(1));
            }
            Event::Key(KeyEvent {
                code: KeyCode::Up, ..
            }) if self.focused_widget == TodoWidget::Todos => {
                self.selected_todo = self.selected_todo.saturating_sub(1);
            }
            Event::Key(KeyEvent {
                code: KeyCode::Down,
                ..
            }) if self.focused_widget == TodoWidget::Buckets => {
                self.selected_bucket = (self.selected_bucket + 1).min(state.bucket_count());
                // bucket_count() ignores the None bucket,
                // so we don't need to subtract 1 here
            }
            Event::Key(KeyEvent {
                code: KeyCode::Up, ..
            }) if self.focused_widget == TodoWidget::Buckets => {
                self.selected_bucket = self.selected_bucket.saturating_sub(1);
            }
            Event::Key(KeyEvent {
                code: KeyCode::Left,
                ..
            }) => {
                if self.selected_todo > 0 && self.selected_todo < state.todo_count() {
                    let _ = state.swap_todos(self.selected_todo, self.selected_todo - 1);
                    self.selected_todo -= 1;
                }
            }
            Event::Key(KeyEvent {
                code: KeyCode::Right,
                ..
            }) => {
                if state.todo_count() > 1 && self.selected_todo < state.todo_count() - 1 {
                    let _ = state.swap_todos(self.selected_todo, self.selected_todo + 1);
                    self.selected_todo += 1;
                }
            }
            Event::Key(KeyEvent {
                code: KeyCode::Char('1'),
                ..
            }) => {
                if self.focused_widget != TodoWidget::TodoInput {
                    return WindowActionResult::FirstWindow;
                }
            }
            Event::Key(KeyEvent {
                code: KeyCode::Char('2'),
                ..
            }) => {
                if self.focused_widget != TodoWidget::TodoInput {
                    return WindowActionResult::SecondWindow;
                }
            }
            Event::Key(KeyEvent {
                code: KeyCode::Char('3'),
                ..
            }) => {
                if self.focused_widget != TodoWidget::TodoInput {
                    return WindowActionResult::ThirdWindow;
                }
            }

            _ => {
                if self.focused_widget == TodoWidget::TodoInput {
                    self.todo_input.handle_event(event);
                } else if self.focused_widget == TodoWidget::BucketInput {
                    self.bucket_input.handle_event(event);
                }
            }
        }
        WindowActionResult::Continue
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TodoWidget {
    Todos,
    TodoInput,
    Buckets,
    BucketInput,
}

struct TodoListWidget<'a> {
    is_focused: bool,
    todos: Vec<&'a TodoItem>,
    selected: usize,
    selected_bucket: Option<&'a str>,
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
                .filter(|x| x.bucket() == self.selected_bucket)
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

struct BucketListWidget<'a> {
    is_focused: bool,
    buckets: Vec<Option<&'a str>>,
    selected: usize,
}
impl<'a> Widget for &BucketListWidget<'a> {
    fn render(self, area: Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        let list_style = if !self.is_focused {
            Style::default()
        } else {
            Color::Yellow.into()
        };
        let list_instructions =
            instruction_line(vec![("Scroll Up", "Up"), ("Scroll Down", "Down")]);
        // TODO: update this with better formatting!
        List::new(
            self.buckets
                .iter()
                .map(|x| format!("<{bucket}>", bucket = x.unwrap_or("N/A"),))
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
