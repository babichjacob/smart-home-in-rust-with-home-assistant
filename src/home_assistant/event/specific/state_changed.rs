use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;

use crate::home_assistant::{entity_id::EntityId, state::State};

#[derive(Debug, Clone)]
pub struct Type;

impl<'py> FromPyObject<'py> for Type {
    fn extract_bound(ob: &Bound<'py, PyAny>) -> PyResult<Self> {
        let s = ob.extract::<&str>()?;

        if s == "state_changed" {
            Ok(Type)
        } else {
            Err(PyValueError::new_err(format!(
                "expected a string of value 'state_changed', but got {s}"
            )))
        }
    }
}

#[derive(Debug, FromPyObject)]
#[pyo3(from_item_all)]
pub struct Data<OldAttributes, OldStateContextEvent, NewAttributes, NewStateContextEvent> {
    pub entity_id: EntityId,
    pub old_state: Option<State<OldAttributes, OldStateContextEvent>>,
    pub new_state: Option<State<NewAttributes, NewStateContextEvent>>,
}

/// A state changed event is fired when on state write the state is changed.
pub type Event<OldAttributes, OldStateContextEvent, NewAttributes, NewStateContextEvent, Context> =
    super::super::event::Event<
        Type,
        Data<OldAttributes, OldStateContextEvent, NewAttributes, NewStateContextEvent>,
        Context,
    >;
