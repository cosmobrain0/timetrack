mod state;
mod todo;
mod track;

use chrono::Utc;
use clap::{Parser, Subcommand};
use clap_derive::Args;
use state::{State, StateBuilder};
use todo::*;
use track::*;

#[derive(Parser, Debug)]
struct Cli {
    #[command(subcommand)]
    command: Option<SubCommand>,
}

#[derive(Clone, Debug, Subcommand)]
enum SubCommand {
    Add {
        name: String,
        target_minutes: usize,
    },
    List,
    Delete {
        id: usize,
    },
    Start {
        id: usize,
    },
    End,
    Register {
        id: usize,
        minutes: usize,
    },
    Overwrite {
        id: usize,
        minutes: usize,
    },
    Pomo {
        #[arg(default_value_t = 30)]
        minutes: usize,
    },
    #[command(alias = "ct")]
    ChangeTarget {
        id: usize,
        minutes: usize,
    },
    #[command(alias = "td")]
    Todo(TodoArgs),
}

#[derive(Debug, Clone, Args)]
struct TodoArgs {
    #[command(subcommand)]
    command: TodoSubCommand,
}

#[derive(Debug, Clone, Subcommand)]
enum TodoSubCommand {
    List,
    Add {
        name: String,
    },
    Delete {
        id: usize,
    },
    Swap {
        id1: usize,
        id2: usize,
    },
    #[command(alias = "ma")]
    MoveAbove {
        anchor: usize,
        to_move: usize,
    },
    #[command(alias = "mb")]
    MoveBelow {
        anchor: usize,
        to_move: usize,
    },
}

fn main() {
    let args = Cli::parse();

    let mut current_state = load_state().expect("failed to load state");

    match args.command {
        Some(SubCommand::Add {
            name,
            target_minutes,
        }) => add_activity(&mut current_state, name, target_minutes),
        Some(SubCommand::List) => list_activities(&current_state),
        Some(SubCommand::Delete { id }) => del_activity(&mut current_state, id),
        Some(SubCommand::Start { id }) => start_activity(&mut current_state, id),
        Some(SubCommand::End) => end_activity(&mut current_state),
        Some(SubCommand::Register { id, minutes }) => {
            register_time(&mut current_state, id, minutes)
        }
        Some(SubCommand::Overwrite { id, minutes }) => {
            overwrite_time(&mut current_state, id, minutes)
        }
        None => list_recommended_action(&current_state),
        Some(SubCommand::Pomo { minutes }) => {
            pomodoro(&mut current_state, minutes);
        }
        Some(SubCommand::ChangeTarget { id, minutes }) => {
            change_target_time(&mut current_state, id, minutes)
        }
        Some(SubCommand::Todo(TodoArgs { command })) => match command {
            TodoSubCommand::List => list_todo(&current_state),
            TodoSubCommand::Add { name } => add_todo(&mut current_state, name),
            TodoSubCommand::Delete { id } => delete_todo(&mut current_state, id),
            TodoSubCommand::Swap { id1, id2 } => swap_todos(&mut current_state, id1, id2),
            TodoSubCommand::MoveAbove { anchor, to_move } => {
                move_todo_above(&mut current_state, anchor, to_move)
            }
            TodoSubCommand::MoveBelow { anchor, to_move } => {
                move_todo_below(&mut current_state, anchor, to_move)
            }
        },
    };
}

fn load_state() -> Result<State, Box<dyn std::error::Error>> {
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
