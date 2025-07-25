mod state;
mod todo;
mod track;

use chrono::Utc;
use color_eyre::Result;
use ratatui::crossterm::event::{self, Event, KeyCode, KeyEvent};
use ratatui::{DefaultTerminal, Frame};
use state::{State, StateBuilder};
use todo::TodoWindow;

fn main() -> Result<()> {
    color_eyre::install()?;
    let mut terminal = ratatui::init();
    let result = App::new()?.run(&mut terminal);
    ratatui::restore();
    result

    // match args.command {
    //     Some(SubCommand::Add {
    //         name,
    //         target_minutes,
    //     }) => add_activity(&mut current_state, name, target_minutes),
    //     Some(SubCommand::List) => list_activities(&current_state),
    //     Some(SubCommand::Delete { id }) => del_activity(&mut current_state, id),
    //     Some(SubCommand::Start { id }) => start_activity(&mut current_state, id),
    //     Some(SubCommand::End) => end_activity(&mut current_state),
    //     Some(SubCommand::Register { id, minutes }) => {
    //         register_time(&mut current_state, id, minutes)
    //     }
    //     Some(SubCommand::Overwrite { id, minutes }) => {
    //         overwrite_time(&mut current_state, id, minutes)
    //     }
    //     None => list_recommended_action(&current_state),
    //     Some(SubCommand::Pomo { minutes }) => {
    //         pomodoro(&mut current_state, minutes);
    //     }
    //     Some(SubCommand::ChangeTarget { id, minutes }) => {
    //         change_target_time(&mut current_state, id, minutes)
    //     }
    //     Some(SubCommand::Todo(TodoArgs { command })) => match command {
    //         TodoSubCommand::List => list_todo(&current_state),
    //         TodoSubCommand::Add { name } => add_todo(&mut current_state, name),
    //         TodoSubCommand::Delete { id } => delete_todo(&mut current_state, id),
    //         TodoSubCommand::Swap { id1, id2 } => swap_todos(&mut current_state, id1, id2),
    //         TodoSubCommand::MoveAbove { anchor, to_move } => {
    //             move_todo_above(&mut current_state, anchor, to_move)
    //         }
    //         TodoSubCommand::MoveBelow { anchor, to_move } => {
    //             move_todo_below(&mut current_state, anchor, to_move)
    //         }
    //     },
    // };
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AppWindow {
    TodoWindow,
}

struct App {
    state: State,
    exit: bool,
    todo_window: TodoWindow,
    current_window: AppWindow,
}
impl App {
    fn new() -> Result<Self> {
        let state = load_state()?;
        Ok(Self {
            state,
            exit: false,
            todo_window: TodoWindow::new(),
            current_window: AppWindow::TodoWindow,
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
        match self.current_window {
            AppWindow::TodoWindow => self.todo_window.draw(&self.state, frame),
        }
    }

    fn handle_events(&mut self) -> std::io::Result<()> {
        // NOTE: this is blocking!
        let evt = event::read()?;
        if let Event::Key(KeyEvent {
            code: KeyCode::Char('q'),
            ..
        }) = evt
        {
            self.exit = true;
        } else {
            match self.current_window {
                AppWindow::TodoWindow => self.todo_window.handle_key_event(&mut self.state, &evt),
            }
        };
        Ok(())
    }
}

fn load_state() -> Result<State> {
    let home = std::env::var("HOME")?;
    let stored_state: StateBuilder = serde_json::from_str(&std::fs::read_to_string(format!(
        "{home}/.timetrack/state.json"
    ))?)?;
    let state: State = stored_state.into();
    if state.date() == Utc::now().date_naive() {
        Ok(state)
    } else {
        Ok(state.refresh())
    }
    // let mut new_state = State::new();
    // new_state.add_activity("Test".into(), 60);
    // new_state.add_activity("Other".into(), 135);
    // Ok(new_state)
}

mod tests {
    #[test]
    fn notifications_work() {
        use mac_notification_sys::*;
        send_notification(
            "This is a title!",
            Some("this is a really really really really long subtitle"),
            "Here's the body!",
            Some(Notification::new().asynchronous(true).wait_for_click(true)),
        )
        .unwrap();
    }
}
