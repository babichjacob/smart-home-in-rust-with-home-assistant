use pyo3::prelude::*;

use crate::{
    home_assistant::entity_id::EntityId,
    python_utils::{detach, validate_type_by_name},
};

use super::state_object::StateObject;

#[derive(Debug)]
pub struct StateMachine(Py<PyAny>);

impl<'py> FromPyObject<'py> for StateMachine {
    fn extract_bound(ob: &Bound<'py, PyAny>) -> PyResult<Self> {
        // region: Validation
        validate_type_by_name(ob, "StateMachine")?;
        // endregion: Validation

        Ok(Self(detach(ob)))
    }
}

impl StateMachine {
    pub fn get<
        'py,
        State: FromPyObject<'py>,
        Attributes: FromPyObject<'py>,
        ContextEvent: FromPyObject<'py>,
    >(
        &self,
        py: Python<'py>,
        entity_id: EntityId,
    ) -> PyResult<Option<StateObject<State, Attributes, ContextEvent>>> {
        let args = (entity_id.to_string(),);
        let state = self.0.call_method1(py, "get", args)?;
        state.extract(py)
    }
}
