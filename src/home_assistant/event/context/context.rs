use pyo3::prelude::*;

use super::id::Id;

/// The context that triggered something.
#[derive(Debug, FromPyObject)]
pub struct Context<Event> {
    pub id: Id,
    pub user_id: Option<String>,
    pub parent_id: Option<String>,
    /// In order to prevent cycles, the user must decide to pass [`Py<PyAny>`] for the `Event` type here
    /// or for the `Context` type in [`Event`]
    pub origin_event: Event,
}
