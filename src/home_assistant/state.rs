use chrono::{DateTime, Utc};
use pyo3::prelude::*;

use crate::{arbitrary::map::Map, home_assistant::entity_id::EntityId};

use super::event::context::context::Context;

#[derive(Debug, FromPyObject)]
pub struct State {
    pub entity_id: EntityId,
    pub state: String,
    pub attributes: Map,
    pub last_changed: Option<DateTime<Utc>>,
    pub last_reported: Option<DateTime<Utc>>,
    pub last_updated: Option<DateTime<Utc>>,
    pub context: Context,
}
