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
use crate::state::{State, TodoItem};
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
        let bucket = self.get_selected_bucket(state);
        state.get_todos_by_bucket(bucket).nth(self.selected_todo)
    }

    fn get_selected_bucket<'a>(&self, state: &'a State) -> Option<&'a str> {
        if self.selected_todo == 0 {
            None
        } else {
            (self.selected_todo - 1 < state.bucket_count())
                .then(|| state.buckets().remove(self.selected_todo - 1))
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
                selected_bucket: self.get_selected_bucket(state),
            },
            list_area,
        );

        frame.render_widget(
            &BucketListWidget {
                is_focused: self.focused_widget == TodoWidget::Buckets,
                buckets: [None]
                    .into_iter()
                    .chain(
                        state
                            .buckets()
                            .into_iter()
                            .map(|name| (name, state.bucket_size(Some(name)) == 0))
                            .map(Option::Some),
                    )
                    .collect(),
                selected: self.selected_bucket,
                purpose: self.bucket_widget_purpose,
            },
            buckets_area,
        );
    }

    fn handle_event(&mut self, state: &mut State, event: &Event) -> WindowActionResult {
        use KeyCode::*;
        match event {
            Event::Key(KeyEvent {
                code,
                modifiers,
                kind: KeyEventKind::Press | KeyEventKind::Repeat,
                state: _state,
            }) => match (code, modifiers) {
                (code, &KeyModifiers::NONE) => match code {
                    Tab if self.bucket_widget_purpose == BucketWidgetPurpose::Browse => {
                        self.focused_widget = match self.focused_widget {
                            TodoWidget::Todos => TodoWidget::TodoInput,
                            TodoWidget::TodoInput => TodoWidget::Buckets,
                            TodoWidget::Buckets => TodoWidget::BucketInput,
                            TodoWidget::BucketInput => TodoWidget::Todos,
                        }
                    }
                    Enter => {
                        if self.focused_widget == TodoWidget::TodoInput {
                            let bucket = self.get_selected_bucket(state).map(ToString::to_string);
                            state.push_todo(TodoItem::new(
                                self.todo_input.value().to_string(),
                                bucket,
                            ));
                            self.todo_input.reset();
                        } else if self.focused_widget == TodoWidget::BucketInput {
                            state.create_bucket(self.bucket_input.value().to_string());
                            self.bucket_input.reset();
                        } else if self.focused_widget == TodoWidget::Todos {
                            let original_bucket_size =
                                state.bucket_size(self.get_selected_bucket(state));
                            if self.selected_todo < original_bucket_size {
                                let bucket =
                                    self.get_selected_bucket(state).map(ToString::to_string);
                                let _ = state.delete_todo_in_bucket(
                                    self.selected_todo,
                                    bucket.as_ref().map(|x| x.as_str()),
                                );
                                self.selected_todo = self
                                    .selected_todo
                                    .min(original_bucket_size.saturating_sub(2));
                            }
                        } else if self.focused_widget == TodoWidget::Buckets {
                            if let Some(bucket) =
                                self.get_selected_bucket(state).map(ToString::to_string)
                            {
                                state.delete_bucket(bucket.as_str());
                            }
                        }
                    }
                    Char('q')
                        if self.focused_widget != TodoWidget::TodoInput
                            && self.focused_widget != TodoWidget::BucketInput =>
                    {
                        return WindowActionResult::Exit;
                    }
                    Down if self.focused_widget == TodoWidget::Todos => {
                        self.selected_todo = (self.selected_todo + 1)
                            .min(state.bucket_size(self.get_selected_bucket(state)));
                    }
                    Up if self.focused_widget == TodoWidget::Todos => {
                        self.selected_todo = self.selected_todo.saturating_sub(1);
                    }
                    Down if self.focused_widget == TodoWidget::Buckets => {
                        self.selected_bucket = (self.selected_bucket + 1).min(state.bucket_count());
                        self.selected_todo = 0;
                    }
                    Up if self.focused_widget == TodoWidget::Buckets => {
                        self.selected_bucket = self.selected_bucket.saturating_sub(1);
                        self.selected_todo = 0;
                    }
                    Left => {
                        if self.selected_todo > 0
                            && self.selected_todo
                                < state.bucket_size(self.get_selected_bucket(state))
                        {
                            let _ = state.swap_todos(self.selected_todo, self.selected_todo - 1);
                            self.selected_todo -= 1;
                        }
                    }
                    Right => {
                        let bucket_size = state.bucket_size(self.get_selected_bucket(state));
                        if bucket_size > 1 && self.selected_todo < bucket_size - 1 {
                            let _ = state.swap_todos(self.selected_todo, self.selected_todo + 1);
                            self.selected_todo += 1;
                        }
                    }
                    Char(' ') if self.focused_widget == TodoWidget::Todos => {
                        self.focused_widget = TodoWidget::Buckets;
                        self.bucket_widget_purpose = BucketWidgetPurpose::Move(self.selected_todo);
                    }
                    Char(' ') if self.focused_widget == TodoWidget::Buckets => {
                        if let BucketWidgetPurpose::Move(x) = self.bucket_widget_purpose {
                            let bucket = self.get_selected_bucket(state).map(ToString::to_string);
                            if let Some(todo) = state.get_todo_mut(x) {
                                todo.set_bucket(bucket);
                            }
                            self.bucket_widget_purpose = BucketWidgetPurpose::Browse;
                            self.focused_widget = TodoWidget::Todos;
                        }
                    }
                    Char('1') => {
                        if self.focused_widget != TodoWidget::TodoInput {
                            return WindowActionResult::FirstWindow;
                        }
                    }
                    Char('2') => {
                        if self.focused_widget != TodoWidget::TodoInput {
                            return WindowActionResult::SecondWindow;
                        }
                    }
                    Char('3') => {
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
                },
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
            ("Change Bucket", "Space"),
        ]);
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
    buckets: Vec<Option<(&'a str, bool)>>,
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
            BucketWidgetPurpose::Move(_) => vec![
                ("Scroll Up", "Up"),
                ("Scroll Down", "Down"),
                ("Select", "Space"),
            ],
        };
        let list_instructions = instruction_line(list_instructions);
        List::new(
            self.buckets
                .iter()
                .map(|x| {
                    format!(
                        "<{}>",
                        x.map(|(name, deletable)| if deletable {
                            name.dark_gray()
                        } else {
                            name.into()
                        })
                        .unwrap_or("N/A".into())
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
            Block::bordered().title(match self.purpose {
                BucketWidgetPurpose::Browse => " Buckets ",
                BucketWidgetPurpose::Move(_) => " Move to Bucket ",
            })
        } else {
            Block::bordered()
                .title(match self.purpose {
                    BucketWidgetPurpose::Browse => " Buckets ",
                    BucketWidgetPurpose::Move(_) => " Move to Bucket ",
                })
                .title_bottom(list_instructions.centered())
        })
        .render(area, buf);
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BucketWidgetPurpose {
    Browse,
    Move(usize),
}
