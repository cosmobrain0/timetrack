use ratatui::crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
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
use crate::state::{Bucket, DEFAULT_BUCKET_NAME, State, TodoItem};
use crate::{Window, WindowActionResult, instruction_line};

#[derive(Debug)]
pub(crate) struct TodoWindow {
    focused_widget: TodoWidget,
    selected_todo: usize,
    selected_bucket: usize,
    todo_input: Input,
    bucket_input: Input,
    bucket_widget_purpose: BucketWidgetPurpose,
}
impl TodoWindow {
    pub fn new() -> Self {
        Self {
            focused_widget: TodoWidget::Todos,
            selected_todo: 0,
            todo_input: Input::new(String::new()),
            bucket_input: Input::new(String::new()),
            selected_bucket: 0,
            bucket_widget_purpose: BucketWidgetPurpose::Browse,
        }
    }
}
impl TodoWindow {
    fn get_selected_todo<'a>(&self, state: &'a State) -> Option<&'a TodoItem> {
        self.get_selected_bucket(state)
            .todos()
            .nth(self.selected_todo)
    }

    fn get_selected_bucket<'a>(&self, state: &'a State) -> &'a Bucket {
        state
            .get_buckets()
            .nth(self.selected_bucket)
            .expect("self.selected_bucket should be a valid bucket index")
    }

    fn get_selected_bucket_mut<'a>(&self, state: &'a mut State) -> &'a mut Bucket {
        state
            .get_buckets_mut()
            .nth(self.selected_bucket)
            .expect("self.selected_bucket should be a valid bucket index")
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
                selected: self.selected_todo,
                selected_bucket: self.get_selected_bucket(state),
            },
            list_area,
        );

        frame.render_widget(
            &BucketListWidget {
                is_focused: self.focused_widget == TodoWidget::Buckets,
                buckets: state.get_buckets().collect(),
                selected: self.selected_bucket,
                purpose: self.bucket_widget_purpose,
            },
            buckets_area,
        );
    }

    fn handle_event(&mut self, state: &mut State, event: &Event) -> WindowActionResult {
        use KeyCode::*;
        use TodoWidget::*;
        match event {
            Event::Key(KeyEvent {
                code,
                modifiers,
                kind: KeyEventKind::Press | KeyEventKind::Repeat,
                state: _state,
            }) => match (code, modifiers) {
                (code, &KeyModifiers::NONE) => {
                    match (code, self.focused_widget, self.bucket_widget_purpose) {
                        (Tab, _, BucketWidgetPurpose::Browse) => {
                            self.focused_widget = match self.focused_widget {
                                TodoWidget::Todos => TodoWidget::TodoInput,
                                TodoWidget::TodoInput => TodoWidget::Buckets,
                                TodoWidget::Buckets => TodoWidget::BucketInput,
                                TodoWidget::BucketInput => TodoWidget::Todos,
                            }
                        }
                        (Enter, TodoInput, _) => {
                            let bucket = self.get_selected_bucket_mut(state);
                            bucket.push_todo(TodoItem::new(self.todo_input.value().to_string()));
                            self.todo_input.reset();
                        }
                        (Enter, BucketInput, _) => {
                            state.create_bucket(Bucket::new(
                                self.bucket_input.value().to_string(),
                                vec![],
                            ));
                            self.bucket_input.reset();
                        }
                        (Enter, Todos, _) => {
                            if self.selected_todo < self.get_selected_bucket(state).todos().count()
                            {
                                let bucket = self.get_selected_bucket_mut(state);
                                *bucket.todos_mut() = bucket
                                    .todos()
                                    .map(TodoItem::clone)
                                    .take(self.selected_todo)
                                    .chain(
                                        bucket
                                            .todos()
                                            .map(TodoItem::clone)
                                            .skip(self.selected_todo + 1),
                                    )
                                    .collect();
                                self.selected_todo = self
                                    .selected_todo
                                    .min(bucket.todos().count().saturating_sub(1));
                            }
                        }
                        (Enter, Buckets, _) => {
                            if state.delete_bucket(self.selected_bucket) {
                                self.selected_bucket = self.selected_bucket.saturating_sub(1);
                            }
                        }
                        (Char('q'), Todos | Buckets, _) => {
                            return WindowActionResult::Exit;
                        }
                        (Down, Todos, _) => {
                            self.selected_todo = (self.selected_todo + 1).min(
                                self.get_selected_bucket(state)
                                    .todos()
                                    .count()
                                    .saturating_sub(1),
                            );
                        }
                        (Up, Todos, _) => {
                            self.selected_todo = self.selected_todo.saturating_sub(1);
                        }
                        (Down, Buckets, _) => {
                            self.selected_bucket = (self.selected_bucket + 1)
                                .min(state.get_buckets().count().saturating_sub(1));
                            self.selected_todo = 0;
                        }
                        (Up, Buckets, _) => {
                            self.selected_bucket = self.selected_bucket.saturating_sub(1);
                            self.selected_todo = 0;
                        }
                        (Left, Todos, _) => {
                            if self.selected_todo > 0 {
                                self.get_selected_bucket_mut(state)
                                    .todos_mut()
                                    .swap(self.selected_todo, self.selected_todo - 1);
                                self.selected_todo -= 1;
                            }
                        }
                        (Right, Todos, _) => {
                            let bucket_size = self.get_selected_bucket(state).todos().count();
                            if bucket_size > 1 && self.selected_todo < bucket_size - 1 {
                                self.get_selected_bucket_mut(state)
                                    .todos_mut()
                                    .swap(self.selected_todo, self.selected_todo + 1);
                                self.selected_todo += 1;
                            }
                        }
                        (Left, Buckets, BucketWidgetPurpose::Browse) => {
                            if self.selected_bucket > 0 {
                                state
                                    .change_bucket_index(
                                        self.selected_bucket,
                                        self.selected_bucket - 1,
                                    )
                                    .expect("should be able to move bucket");
                                self.selected_bucket -= 1;
                            }
                        }
                        (Right, Buckets, BucketWidgetPurpose::Browse) => {
                            if self.selected_bucket < state.get_buckets().count() - 1 {
                                state
                                    .change_bucket_index(
                                        self.selected_bucket,
                                        self.selected_bucket + 1,
                                    )
                                    .expect("should be able to move bucket");
                                self.selected_bucket += 1;
                            }
                        }
                        (Char(' '), Todos, _) => {
                            if self.selected_todo < self.get_selected_bucket(state).todos().count()
                            {
                                self.focused_widget = TodoWidget::Buckets;
                                self.bucket_widget_purpose = BucketWidgetPurpose::Move {
                                    selected_bucket: self.selected_bucket,
                                    selected_todo: self.selected_todo,
                                };
                            }
                        }
                        (
                            Char(' '),
                            Buckets,
                            BucketWidgetPurpose::Move {
                                selected_bucket,
                                selected_todo,
                            },
                        ) => {
                            let todo_item = state
                                .get_buckets_mut()
                                .nth(selected_bucket)
                                .expect("should be able to get source bucket of move")
                                .todos_mut()
                                .remove(selected_todo);
                            state
                                .get_buckets_mut()
                                .nth(self.selected_bucket)
                                .expect("should be able to get destination bucket for move")
                                .todos_mut()
                                .push(todo_item);
                            self.bucket_widget_purpose = BucketWidgetPurpose::Browse;
                            self.focused_widget = TodoWidget::Todos;
                        }
                        (Char('1'), Todos | Buckets, _) => {
                            return WindowActionResult::FirstWindow;
                        }
                        (Char('2'), Todos | Buckets, _) => {
                            return WindowActionResult::SecondWindow;
                        }
                        (Char('3'), Todos | Buckets, _) => {
                            return WindowActionResult::ThirdWindow;
                        }
                        (_, TodoInput, _) => {
                            self.todo_input.handle_event(event);
                        }
                        (_, BucketInput, _) => {
                            self.bucket_input.handle_event(event);
                        }
                        _ => (),
                    }
                }
                _ => (),
            },
            _ => (),
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
    selected: usize,
    selected_bucket: &'a Bucket,
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
            ("Change Bucket", "Space"),
        ]);
        let bucket = self.selected_bucket.name();
        List::new(
            self.selected_bucket
                .todos()
                .map(|x| format!("<{bucket}> {item}", item = x.item()))
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
    buckets: Vec<&'a Bucket>,
    selected: usize,
    purpose: BucketWidgetPurpose,
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
        let list_instructions = match self.purpose {
            BucketWidgetPurpose::Browse => vec![("Scroll Up", "Up"), ("Scroll Down", "Down")],
            BucketWidgetPurpose::Move { .. } => vec![
                ("Scroll Up", "Up"),
                ("Scroll Down", "Down"),
                ("Select", "Space"),
            ],
        };
        let list_instructions = instruction_line(list_instructions);
        List::new(
            self.buckets
                .iter()
                .enumerate()
                .map(|(i, x)| {
                    (
                        x.todos().count() == 0 && x.name() != DEFAULT_BUCKET_NAME,
                        self.is_focused && i == self.selected,
                        format!("<{}>", x.name()),
                    )
                })
                .map(|(deletable, focused, x)| match (focused, deletable) {
                    (true, true) => x.on_dark_gray().blue().bold(),
                    (true, false) => x.blue().bold(),
                    (false, true) => x.dark_gray(),
                    (false, false) => x.into(),
                }),
        )
        .style(list_style)
        .block(if !self.is_focused {
            Block::bordered().title(match self.purpose {
                BucketWidgetPurpose::Browse => " Buckets ",
                BucketWidgetPurpose::Move { .. } => " Move to Bucket ",
            })
        } else {
            Block::bordered()
                .title(match self.purpose {
                    BucketWidgetPurpose::Browse => " Buckets ",
                    BucketWidgetPurpose::Move { .. } => " Move to Bucket ",
                })
                .title_bottom(list_instructions.centered())
        })
        .render(area, buf);
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BucketWidgetPurpose {
    Browse,
    Move {
        selected_bucket: usize,
        selected_todo: usize,
    },
}
