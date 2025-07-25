use ratatui::crossterm::event::{Event, KeyCode, KeyEvent};
use ratatui::widgets::List;
use ratatui::{
    Frame,
    layout::{Constraint, Layout},
    style::{Color, Style},
    widgets::{Block, Paragraph},
};
use tui_input::{Input, backend::crossterm::EventHandler};

use crate::state::{State, TodoDeletionError, TodoSwapError};

pub(crate) fn list_todo(current_state: &crate::state::State) {
    if current_state.get_todos().count() == 0 {
        println!("Your todo list is empty!");
    } else {
        for todo in current_state
            .get_todos()
            .enumerate()
            .map(|(i, todo)| State::format_todo(i, todo.clone()))
        {
            println!("{todo}");
        }
    }
}

pub(crate) fn add_todo(current_state: &mut crate::state::State, name: String) {
    current_state.push_todo(name);
    println!("Added todo item!");
}

pub(crate) fn delete_todo(current_state: &mut crate::state::State, id: usize) {
    match current_state.delete_todo(id) {
        Ok(_) => println!("Deleted Todo {id}", id = State::format_todo_id(id)),
        Err(TodoDeletionError::InvalidId) => println!("Todo Id is invalid!"),
    }
}

pub(crate) fn swap_todos(current_state: &mut crate::state::State, id1: usize, id2: usize) {
    match current_state.swap_todos(id1, id2) {
        Ok(_) => println!(
            "Swapped todos {id1} and {id2}!",
            id1 = State::format_todo_id(id1),
            id2 = State::format_todo_id(id2)
        ),
        Err(TodoSwapError::EqualIds) => println!("Those ids are the same!"),
        Err(TodoSwapError::FirstInvalid) => println!("The first ID is invalid!"),
        Err(TodoSwapError::SecondInvalid) => println!("The second ID is invalid!"),
    }
}

pub(crate) fn move_todo_above(
    current_state: &mut crate::state::State,
    anchor: usize,
    to_move: usize,
) {
    let total_todos = current_state.todo_count();
    if anchor >= total_todos {
        println!("Anchor is invalid!");
    } else if to_move >= total_todos {
        println!("ID of Todo to move is invalid!");
    } else if anchor == to_move {
        println!("Can't move a Todo above itself!");
    } else {
        let to_move_todo = current_state.delete_todo(to_move).unwrap();
        current_state
            .insert_todo(
                to_move_todo,
                if to_move > anchor { anchor } else { anchor - 1 },
            )
            .unwrap();
    }
}

pub(crate) fn move_todo_below(
    current_state: &mut crate::state::State,
    anchor: usize,
    to_move: usize,
) {
    let total_todos = current_state.todo_count();
    if anchor >= total_todos {
        println!("Anchor is invalid!");
    } else if to_move >= total_todos {
        println!("ID of Todo to move is invalid!");
    } else if anchor == to_move {
        println!("Can't move a Todo above itself!");
    } else {
        let to_move_todo = current_state.delete_todo(to_move).unwrap();
        current_state
            .insert_todo(
                to_move_todo,
                if to_move > anchor { anchor + 1 } else { anchor },
            )
            .unwrap();
    }
}

pub(crate) struct TodoWindow {
    input_focused: bool,
    list_page: usize,
    input: Input,
}
impl TodoWindow {
    pub fn new() -> Self {
        Self {
            input_focused: false,
            list_page: 0,
            input: Input::new(String::new()),
        }
    }

    pub fn draw(&self, state: &State, frame: &mut Frame) {
        let [list_area, input_area] =
            Layout::vertical([Constraint::Min(3), Constraint::Length(3)]).areas(frame.area());
        let width = frame.area().width - 3;
        let scroll = self.input.visual_scroll(width as usize);
        let style = if self.input_focused {
            Color::Yellow.into()
        } else {
            Style::default()
        };
        let input = Paragraph::new(self.input.value())
            .style(style)
            .scroll((0, scroll as u16))
            .block(Block::bordered().title("New Todo"));
        frame.render_widget(input, input_area);

        let list = List::new(state.get_todos().map(|x| format!("- {x}")))
            .block(Block::bordered().title("Todo Items"));
        frame.render_widget(list, list_area);
    }

    pub fn handle_key_event(&mut self, state: &mut State, event: &Event) {
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
                state.push_todo(self.input.value().to_string());
                self.input.reset();
            }
            _ => {
                if self.input_focused {
                    self.input.handle_event(event);
                }
            }
        }
    }
}
