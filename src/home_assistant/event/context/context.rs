use pyo3::prelude::*;

use crate::home_assistant::event::event::Event;

use super::id::Id;

/// The context that triggered something.
#[derive(Debug, FromPyObject)]
pub struct Context {
    pub id: Id,
    pub user_id: Option<String>,
    pub parent_id: Option<String>,
    /// In order to prevent cycles, the user must extract this to an [`Event<Arbitrary>`](super::event::Event) themself (or even specify a specific type parameter!)
    origin_event: Py<PyAny>,
}

impl Context {
    pub fn origin_event<'py, Type: FromPyObject<'py>, Data: FromPyObject<'py>>(
        &self,
        py: Python<'py>,
    ) -> PyResult<Event<Type, Data>> {
        self.origin_event.extract(py)
    }
}
