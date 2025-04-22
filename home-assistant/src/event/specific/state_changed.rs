use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;

use crate::{entity_id::EntityId, state_object::StateObject};

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
pub struct Data<
    OldState,
    OldAttributes,
    OldStateContextEvent,
    NewState,
    NewAttributes,
    NewStateContextEvent,
> {
    pub entity_id: EntityId,
    pub old_state: Option<StateObject<OldState, OldAttributes, OldStateContextEvent>>,
    pub new_state: Option<StateObject<NewState, NewAttributes, NewStateContextEvent>>,
}

/// A state changed event is fired when on state write the state is changed.
pub type Event<
    OldState,
    OldAttributes,
    OldStateContextEvent,
    NewState,
    NewAttributes,
    NewStateContextEvent,
    Context,
> = super::super::event::Event<
    Type,
    Data<
        OldState,
        OldAttributes,
        OldStateContextEvent,
        NewState,
        NewAttributes,
        NewStateContextEvent,
    >,
    Context,
>;
