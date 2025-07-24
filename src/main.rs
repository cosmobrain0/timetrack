use std::{
    io::Write,
    sync::{Arc, atomic::AtomicBool},
};

use chrono::{Duration, Utc};
use clap::{Parser, Subcommand};
use crossterm::{cursor, execute, style};
use state::{Activity, StartActivityError, State, StateBuilder};

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
    };
}

fn pomodoro(current_state: &mut State, minutes: usize) {
    let (activity, session_minutes) = match find_recommended_action(current_state) {
        Ok(activity) => (
            activity,
            activity
                .target_minutes()
                .saturating_sub(activity.acheived_minutes())
                .min(minutes),
        ),
        Err(FindRecommendedActionError::NoMoreTasks) => {
            println!("You have no more tasks! You're done!");
            return;
        }
        Err(FindRecommendedActionError::Ongoing(activity)) => {
            println!(
                "Stop your current task before running pomo!\n{}",
                current_state.format_activity(activity, None)
            );
            return;
        }
        Err(FindRecommendedActionError::OngoingCompleted(activity)) => {
            println!(
                "Stop your current task!\n{}",
                current_state.format_activity(activity, None)
            );
            return;
        }
    };

    let interrupted = Arc::new(AtomicBool::new(false));

    let interrupted_clone = Arc::clone(&interrupted);
    ctrlc::set_handler(move || {
        interrupted_clone.store(true, std::sync::atomic::Ordering::Relaxed);
    })
    .expect("should be able to set ctrlc handler!");

    println!("Work on this task for {session_minutes}min!");
    let activity_id = activity.id();
    let duration = Duration::seconds(session_minutes as i64 * 60);
    current_state
        .start_activity_pomo(activity_id, Some(session_minutes))
        .expect("should be able to start activity");
    let start = Utc::now();
    let end = start + duration;
    let activity = current_state
        .activity_by_id(activity_id)
        .expect("should be able to get ID of started activity");
    println!("{}", current_state.format_activity(activity, None));

    const TIMER_LENGTH: usize = 10;
    let mut stdout = std::io::stdout();
    while Utc::now() < end && !interrupted.load(std::sync::atomic::Ordering::Relaxed) {
        let length = (TIMER_LENGTH as f64 * (Utc::now() - start).num_seconds() as f64
            / duration.num_seconds() as f64)
            .floor() as usize;
        execute!(
            stdout,
            cursor::MoveToColumn(0),
            style::Print(format!(
                "[{bars}>{padding}]",
                bars = "=".repeat(length),
                padding = " ".repeat(TIMER_LENGTH.saturating_sub(length))
            ))
        )
        .expect("should be able to draw progress bar");
        stdout.flush().unwrap();
        std::thread::sleep(std::time::Duration::from_millis(50));
    }
    println!("\n");

    if interrupted.load(std::sync::atomic::Ordering::Relaxed) {
        let time_left = ((end - Utc::now()).num_seconds() as f64 / 60.0).ceil() as usize;
        println!("You've ended the session {time_left}min early!");
    } else {
        println!("Stop working!");
    }
    current_state
        .end_activity(true)
        .expect("should be able to end activity in pomo!");

    let activity = current_state
        .activity_by_id(activity_id)
        .expect("should be able to get ID of finished activity");
    let activity_name = activity.name();
    println!("{}", current_state.format_activity(activity, None));

    mac_notification_sys::send_notification(
        "Pomodoro Session Over!!",
        Some(activity_name),
        &format!("You've worked for {session_minutes}min on {activity_name}!"),
        Some(
            mac_notification_sys::Notification::new()
                .asynchronous(true)
                .wait_for_click(true),
        ),
    )
    .expect("should be able to send notification!");
}

fn overwrite_time(current_state: &mut State, id: usize, minutes: usize) {
    if let Some(id) = current_state.get_by_raw_id(id) {
        current_state.overwrite_time(id, minutes).unwrap();
        println!("Set acheived minutes of {id} to {minutes}min!");
    } else {
        println!("There is no activity with that id!");
    }
}

fn register_time(current_state: &mut State, id: usize, minutes: usize) {
    if let Some(id) = current_state.get_by_raw_id(id) {
        current_state.add_time(id, minutes).unwrap();
        println!("Added {minutes}min to {id}!");
    } else {
        println!("There is no activity with that id!");
    }
}

fn end_activity(current_state: &mut State) {
    match current_state.end_activity(false) {
        Ok(()) => println!("Ended activity!"),
        Err(state::EndActivityError::NoCurrentActivity) => {
            println!("There is no ongoing activity!")
        }
        Err(state::EndActivityError::PomoOngoing) => {
            println!("You must cancel the ongoing pomo session first!");
        }
    }
}

fn start_activity(current_state: &mut State, id: usize) {
    if let Some(id) = current_state.get_by_raw_id(id) {
        match current_state.start_activity(id) {
            Ok(()) => println!("Started activity {id}!"),
            Err(StartActivityError::AlreadyOngoing) => {
                println!("There is already an ongoing activity!")
            }
            Err(state::StartActivityError::InvalidId) => {
                println!("There is no activity with that ID!")
            } // this should be unreachable but whatever
        }
    } else {
        println!("There is no activity with that ID!");
    }
}

fn del_activity(current_state: &mut State, id: usize) {
    if let Some(id) = current_state.get_by_raw_id(id) {
        match current_state.delete(id) {
            Ok(()) => println!("Deleted activity {id}!"),
            Err(state::DeletionError::PomoOngoing) => {
                println!("You must end the currently ongoing pomo session first!")
            }
            Err(state::DeletionError::InvalidId) => unreachable!(),
        }
    } else {
        println!("There is no activity with id {id}!");
    }
}

fn list_activities(current_state: &State) {
    let current_activity = current_state.current_id();
    let current_duration = current_state.current_session_duration();
    let mut activities = current_state.activities();
    activities.sort_by_key(|x| {
        x.acheived_minutes()
            + current_activity
                .is_some_and(|id| id == x.id())
                .then(|| current_duration.unwrap().num_minutes().max(0) as usize)
                .unwrap_or(0)
    });
    let activities = activities;
    let max_name_length = activities
        .iter()
        .map(|x| x.name().chars().count() + 1)
        .max()
        .unwrap_or(1);
    if activities.is_empty() {
        println!("No activities!");
    }
    for activity in activities {
        println!(
            "{}",
            current_state.format_activity(activity, Some(max_name_length))
        );
    }
}

enum FindRecommendedActionError<'a> {
    NoMoreTasks,
    Ongoing(&'a Activity),
    OngoingCompleted(&'a Activity),
}

fn find_recommended_action(
    current_state: &State,
) -> Result<&Activity, FindRecommendedActionError<'_>> {
    if let Some(current_task) = current_state.current_activity() {
        if current_task.acheived_minutes()
            + current_state
                .current_session_duration()
                .unwrap()
                .num_minutes()
                .max(0) as usize
            >= current_task.target_minutes()
        {
            Err(FindRecommendedActionError::OngoingCompleted(current_task))
        } else {
            Err(FindRecommendedActionError::Ongoing(current_task))
        }
    } else if let Some(activity) = current_state
        .activities()
        .iter()
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

fn list_recommended_action(current_state: &State) {
    match find_recommended_action(current_state) {
        Ok(activity) => println!("{}", current_state.format_activity(activity, None)),
        Err(FindRecommendedActionError::NoMoreTasks) => {
            println!("There are no more tasks! You're done!")
        }
        Err(FindRecommendedActionError::OngoingCompleted(activity)) => {
            let task_formatted = current_state.format_activity(activity, None);
            println!("You're currently doing:\n{task_formatted}\nStop the current task!");
        }
        Err(FindRecommendedActionError::Ongoing(activity)) => {
            let task_formatted = current_state.format_activity(activity, None);
            println!("Continue with your current task!\n{task_formatted}");
        }
    }
}

fn add_activity(current_state: &mut State, name: String, target_minutes: usize) {
    let new_activity_id = current_state.add_activity(name.clone(), target_minutes);
    println!("Added activity {new_activity_id}: {name} with target {target_minutes}min");
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

mod state {
    use std::fmt::Display;

    use chrono::{DateTime, NaiveDate, TimeDelta, Utc};
    use colored::Colorize;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct StateBuilder {
        pub date: Option<NaiveDate>,
        pub activities: Option<Vec<Activity>>,
        pub next_activity_id: Option<usize>,
        pub current: Option<CurrentActionInfo>,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct State {
        date: NaiveDate,
        activities: Vec<Activity>,
        next_activity_id: usize,
        current: Option<CurrentActionInfo>,
    }
    impl From<StateBuilder> for State {
        fn from(value: StateBuilder) -> Self {
            Self {
                next_activity_id: value.next_activity_id.unwrap_or_else(|| {
                    value
                        .activities
                        .as_ref()
                        .map(|activities| activities.iter().map(|x| x.id.0).max().unwrap_or(0) + 1)
                        .unwrap_or(0)
                }),
                activities: value.activities.unwrap_or_default(),
                date: value.date.unwrap_or_else(|| Utc::now().date_naive()),
                current: value.current,
            }
        }
    }
    impl State {
        pub fn refresh(self) -> Self {
            Self {
                date: Utc::now().date_naive(),
                activities: self
                    .activities
                    .clone()
                    .into_iter()
                    .map(|x| x.with_acheived_reset())
                    .collect(),
                next_activity_id: self.next_activity_id,
                current: self.current,
            }
        }

        pub fn add_activity(&mut self, name: String, target_minutes: usize) -> ActivityId {
            let activity = Activity {
                target_minutes,
                acheived_minutes: 0,
                name,
                id: self.new_activity_id(),
            };
            let id = activity.id;
            self.activities.push(activity);
            self.save_state().unwrap();
            id
        }

        fn new_activity_id(&mut self) -> ActivityId {
            let id = ActivityId(self.next_activity_id + 1);
            self.next_activity_id += 1;
            self.save_state().unwrap();
            id
        }

        pub fn activities(&self) -> Vec<&Activity> {
            self.activities.iter().collect()
        }

        pub fn get_by_raw_id(&self, id: usize) -> Option<ActivityId> {
            self.activities
                .iter()
                .find_map(|activity| (activity.id == ActivityId(id)).then_some(ActivityId(id)))
        }

        fn get_index_by_id(&self, id: ActivityId) -> Option<usize> {
            self.activities
                .iter()
                .enumerate()
                .find_map(|(i, activity)| (activity.id == id).then_some(i))
        }

        pub fn delete(&mut self, id: ActivityId) -> Result<(), DeletionError> {
            if let Some(index) = self.get_index_by_id(id) {
                if self
                    .current
                    .is_some_and(|current_action| id == current_action.activity_id)
                {
                    if let Err(EndActivityError::PomoOngoing) = self.end_activity(false) {
                        return Err(DeletionError::PomoOngoing);
                    }
                }
                self.activities.remove(index);
                self.save_state().unwrap();
                Ok(())
            } else {
                Err(DeletionError::InvalidId)
            }
        }

        pub fn current_id(&self) -> Option<ActivityId> {
            self.current.map(|x| x.activity_id)
        }

        pub fn start_activity(&mut self, id: ActivityId) -> Result<(), StartActivityError> {
            self.start_activity_pomo(id, None)
        }

        pub fn start_activity_pomo(
            &mut self,
            id: ActivityId,
            pomo_minutes: Option<usize>,
        ) -> Result<(), StartActivityError> {
            if self.current.is_some() {
                Err(StartActivityError::AlreadyOngoing)
            } else if self.get_index_by_id(id).is_some() {
                self.current = Some(CurrentActionInfo::new(id, Utc::now(), pomo_minutes));
                self.save_state().unwrap();
                Ok(())
            } else {
                Err(StartActivityError::InvalidId)
            }
        }

        pub fn end_activity(&mut self, override_pomo: bool) -> Result<(), EndActivityError> {
            if let Some(current) = self.current {
                if !override_pomo && current.pomo_minutes.is_some() {
                    return Err(EndActivityError::PomoOngoing);
                }
                let now = Utc::now();
                let delta = now - current.start_time;
                if let Some(index) = self.get_index_by_id(current.activity_id) {
                    // this should always be the case
                    self.activities[index].acheived_minutes += delta.num_minutes().max(0) as usize;
                }
                self.current = None;
                self.save_state().unwrap();
                Ok(())
            } else {
                Err(EndActivityError::NoCurrentActivity)
            }
        }

        pub fn add_time(&mut self, id: ActivityId, minutes: usize) -> Result<(), ()> {
            if let Some(index) = self.get_index_by_id(id) {
                self.activities[index].acheived_minutes += minutes;
                self.save_state().unwrap();
                Ok(())
            } else {
                Err(())
            }
        }

        pub fn overwrite_time(&mut self, id: ActivityId, minutes: usize) -> Result<(), ()> {
            if let Some(index) = self.get_index_by_id(id) {
                self.activities[index].acheived_minutes = minutes;
                self.save_state().unwrap();
                Ok(())
            } else {
                Err(())
            }
        }

        pub fn date(&self) -> NaiveDate {
            self.date
        }

        pub fn current_task_minutes(&self) -> Option<usize> {
            self.current.map(|current_activity| {
                (Utc::now() - current_activity.start_time)
                    .num_minutes()
                    .max(0) as usize
            })
        }

        pub fn activity_by_id(&self, id: ActivityId) -> Option<&Activity> {
            self.activities.iter().find(|activity| activity.id == id)
        }

        pub fn current_session_duration(&self) -> Option<TimeDelta> {
            self.current
                .map(|current_activity| Utc::now() - current_activity.start_time)
        }

        pub fn current_activity(&self) -> Option<&Activity> {
            self.current_id().map(|id| self.activity_by_id(id).unwrap())
        }

        pub fn format_activity(
            &self,
            activity: &Activity,
            max_name_length: Option<usize>,
        ) -> String {
            let pad = |s: &str| {
                if let Some(max_name_length) = max_name_length {
                    let current_length = s.chars().count();
                    s.to_string() + &" ".repeat(max_name_length - current_length)
                } else {
                    s.to_string()
                }
            };
            let ongoing = self.current_id().is_some_and(|x| x == activity.id());
            let acheived = activity.acheived_minutes()
                + if ongoing {
                    self.current_task_minutes().unwrap_or(0)
                } else {
                    0
                };
            let target = activity.target_minutes();
            let remaining = target.saturating_sub(acheived);
            let highlight_colour = if ongoing {
                if acheived < target { "blue" } else { "red" }
            } else if acheived < target {
                "white"
            } else {
                "green"
            };
            format!(
                "{id} {status} {name} {remaining} / {target}",
                id = activity.id().to_string().color(highlight_colour),
                name = pad(activity.name()),
                status = if ongoing {
                    if acheived < target {
                        "ONGOING "
                    } else {
                        "OVERWORK"
                    }
                } else if acheived < target {
                    "NOT DONE"
                } else {
                    "COMPLETE"
                }
                .color(highlight_colour)
            )
        }

        fn save_state(&self) -> Result<(), Box<dyn std::error::Error>> {
            let home = std::env::var("HOME")?;
            std::fs::write(
                format!("{home}/.timetrack/state.json"),
                serde_json::to_string(self).expect("should be able to convert to string"),
            )?;
            Ok(())
        }
    }
    impl Drop for State {
        fn drop(&mut self) {
            self.save_state().expect("should be able to save state");
        }
    }

    #[derive(Debug, Clone, Copy, Serialize, Deserialize)]
    pub struct CurrentActionInfo {
        activity_id: ActivityId,
        start_time: DateTime<Utc>,
        pomo_minutes: Option<usize>,
    }
    impl CurrentActionInfo {
        fn new(
            activity_id: ActivityId,
            start_time: DateTime<Utc>,
            pomo_minutes: Option<usize>,
        ) -> Self {
            Self {
                activity_id,
                start_time,
                pomo_minutes,
            }
        }
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum StartActivityError {
        /// There is already another activity in progress
        AlreadyOngoing,
        InvalidId,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum EndActivityError {
        PomoOngoing,
        NoCurrentActivity,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum DeletionError {
        PomoOngoing,
        InvalidId,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct Activity {
        target_minutes: usize,
        acheived_minutes: usize,
        name: String,
        id: ActivityId,
    }
    impl Activity {
        pub fn name(&self) -> &str {
            &self.name
        }

        pub fn id(&self) -> ActivityId {
            self.id
        }

        pub fn target_minutes(&self) -> usize {
            self.target_minutes
        }

        pub fn acheived_minutes(&self) -> usize {
            self.acheived_minutes
        }

        fn with_acheived_reset(self) -> Self {
            Self {
                acheived_minutes: 0,
                ..self
            }
        }
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    pub struct ActivityId(usize);
    impl Display for ActivityId {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "[{id}]", id = self.0)
        }
    }
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
