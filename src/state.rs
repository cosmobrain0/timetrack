use std::fmt::Display;

use chrono::{DateTime, NaiveDate, TimeDelta, Utc};
use ratatui::{style::Stylize, text::Line};
use serde::{Deserialize, Serialize};
pub use todos_and_buckets::{Bucket, TodoItem, TodoItemOld};

use crate::stored_state_file_path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateBuilder {
    pub date: Option<NaiveDate>,
    pub activities: Option<Vec<Activity>>,
    pub next_activity_id: Option<usize>,
    pub current: Option<CurrentActionInfo>,
    /// NOTE: this was used in an older version, before buckets
    pub todo: Option<Vec<String>>,
    pub todo_v2: Option<Vec<TodoItemOld>>,
    pub buckets: Option<Vec<String>>,
    pub buckets_v2: Option<Vec<Bucket>>,
}

pub const DEFAULT_BUCKET_NAME: &str = "N/A";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct State {
    date: NaiveDate,
    activities: Vec<Activity>,
    next_activity_id: usize,
    current: Option<CurrentActionInfo>,
    buckets: Vec<Bucket>,
}
impl From<StateBuilder> for State {
    fn from(value: StateBuilder) -> Self {
        let mut buckets = value.buckets_v2.unwrap_or_default();
        for name in value.buckets.unwrap_or_default() {
            if buckets.iter().map(|x| x.name()).all(|x| x != name.as_str()) {
                buckets.push(Bucket::new(name, vec![]));
            }
        }
        // REQUIREMENT: `buckets` must contain an `N/A` bucket
        let default_bucket = if let Some(default_bucket) =
            buckets.iter_mut().find(|x| x.name() == DEFAULT_BUCKET_NAME)
        {
            default_bucket
        } else {
            buckets.push(Bucket::new(String::from(DEFAULT_BUCKET_NAME), vec![]));
            let last_index = buckets.len() - 1;
            &mut buckets[last_index]
        };

        for old_todo in value.todo.unwrap_or_default() {
            default_bucket.push_todo(TodoItem::new(old_todo));
        }

        for todo in value.todo_v2.unwrap_or_default() {
            if let Some(target_bucket) = buckets.iter_mut().find(|x| {
                x.name()
                    == todo
                        .bucket
                        .as_ref()
                        .map(|x| x.as_str())
                        .unwrap_or(DEFAULT_BUCKET_NAME)
            }) {
                target_bucket.push_todo(TodoItem::new(todo.item));
            }
        }

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
            buckets,
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
            buckets: self.buckets.clone(),
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
        id
    }

    fn new_activity_id(&mut self) -> ActivityId {
        let id = ActivityId(self.next_activity_id + 1);
        self.next_activity_id += 1;
        id
    }

    pub fn activities(&self) -> std::slice::Iter<'_, Activity> {
        self.activities.iter()
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
            Ok(())
        } else {
            Err(EndActivityError::NoCurrentActivity)
        }
    }

    pub fn add_time(&mut self, id: ActivityId, minutes: usize) -> Result<(), ()> {
        if let Some(index) = self.get_index_by_id(id) {
            self.activities[index].acheived_minutes += minutes;
            Ok(())
        } else {
            Err(())
        }
    }

    pub fn overwrite_time(&mut self, id: ActivityId, minutes: usize) -> Result<(), ()> {
        if let Some(index) = self.get_index_by_id(id) {
            self.activities[index].acheived_minutes = minutes;
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

    pub fn format_activity(&self, activity: &Activity, max_name_length: Option<usize>) -> Line {
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
        let status = {
            let status = if ongoing {
                if acheived < target {
                    "ONGOING "
                } else {
                    "OVERWORK"
                }
            } else if acheived < target {
                "NOT DONE"
            } else {
                "COMPLETE"
            };

            if ongoing {
                if acheived < target {
                    status.blue()
                } else {
                    status.red()
                }
            } else if acheived < target {
                status.into()
            } else {
                status.green()
            }
        };
        Line::from(vec![
            status,
            " ".into(),
            pad(activity.name()).into(),
            " ".into(),
            remaining.to_string().into(),
            " / ".into(),
            target.to_string().into(),
        ])
    }

    fn save_state(&self) -> Result<(), Box<dyn std::error::Error>> {
        std::fs::write(
            stored_state_file_path()?,
            serde_json::to_string(&StateBuilder {
                date: Some(self.date),
                activities: Some(self.activities.clone()),
                next_activity_id: Some(self.next_activity_id),
                current: self.current,
                todo: None,
                buckets_v2: Some(self.buckets.clone()),
                todo_v2: None,
                buckets: None,
            })
            .expect("should be able to convert to string"),
        )?;
        Ok(())
    }

    pub(crate) fn get_by_raw_id_mut(&mut self, id: usize) -> Option<&mut Activity> {
        self.activities
            .iter_mut()
            .find(|activity| activity.id == ActivityId(id))
    }

    pub(crate) fn get_by_id_mut(&mut self, id: ActivityId) -> Option<&mut Activity> {
        self.get_by_raw_id_mut(id.0)
    }

    pub(crate) fn activities_count(&self) -> usize {
        self.activities.len()
    }

    pub(crate) fn pomo_minutes(&self) -> Option<usize> {
        self.current.as_ref().and_then(|x| x.pomo_minutes)
    }
}
impl State {
    pub(crate) fn get_buckets(&self) -> impl Iterator<Item = &Bucket> {
        self.buckets.iter()
    }

    pub(crate) fn get_buckets_mut(&mut self) -> impl Iterator<Item = &mut Bucket> {
        self.buckets.iter_mut()
    }

    /// Returns true if the bucket was added,
    /// and false if its todos were combined
    /// with an already existing bucket
    pub(crate) fn create_bucket(&mut self, bucket: Bucket) -> bool {
        if let Some(stored_bucket) = self.get_buckets_mut().find(|x| *x == &bucket) {
            stored_bucket
                .todos_mut()
                .extend(bucket.todos().map(TodoItem::clone));
            false
        } else {
            self.buckets.push(bucket);
            true
        }
    }

    pub(crate) fn delete_bucket(&mut self, index: usize) -> bool {
        if self.buckets.len() > index
            && self.buckets[index].name() != DEFAULT_BUCKET_NAME
            && self.buckets[index].todos().count() == 0
        {
            self.buckets.remove(index);
            true
        } else {
            false
        }
    }

    pub(crate) fn change_bucket_index(
        &mut self,
        original_index: usize,
        new_index: usize,
    ) -> Result<(), BucketSwapError> {
        if original_index >= self.buckets.len() {
            Err(BucketSwapError::InvalidSelection)
        } else if new_index >= self.buckets.len() {
            Err(BucketSwapError::InvalidTargetIndex)
        } else {
            let bucket = self.buckets.remove(original_index);
            self.buckets.insert(new_index, bucket);
            Ok(())
        }
    }
}
impl Drop for State {
    fn drop(&mut self) {
        self.save_state().expect("should be able to save state");
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TodoDeletionError {
    InvalidId,
    InvalidIdOrBucket,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TodoSwapError {
    SecondInvalid,
    FirstInvalid,
    EqualIds,
    InvalidBucket,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BucketSwapError {
    InvalidSelection,
    InvalidTargetIndex,
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

    pub(crate) fn set_target_minutes(&mut self, target_minutes: usize) {
        self.target_minutes = target_minutes;
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ActivityId(usize);
impl Display for ActivityId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{id}]", id = self.0)
    }
}

mod todos_and_buckets {
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct TodoItemOld {
        pub item: String,
        pub bucket: Option<String>,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct TodoItem(String);
    impl TodoItem {
        pub fn new(item: String) -> Self {
            Self(item)
        }

        pub fn item(&self) -> &str {
            &self.0
        }
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct Bucket {
        name: String,
        todos: Vec<TodoItem>,
    }
    impl std::hash::Hash for Bucket {
        fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
            self.name.hash(state);
        }
    }
    impl Bucket {
        pub fn new(name: String, todos: Vec<TodoItem>) -> Self {
            Self { name, todos }
        }

        pub fn name(&self) -> &str {
            &self.name
        }
        pub fn todos(&self) -> impl Iterator<Item = &TodoItem> {
            self.todos.iter()
        }

        pub fn set_name(&mut self, name: String) {
            self.name = name;
        }

        pub fn todos_mut(&mut self) -> &mut Vec<TodoItem> {
            &mut self.todos
        }

        pub(crate) fn push_todo(&mut self, todo: TodoItem) {
            self.todos.push(todo);
        }
    }
    impl PartialEq for Bucket {
        fn eq(&self, other: &Self) -> bool {
            self.name == other.name
        }
    }
    impl Eq for Bucket {}
}
