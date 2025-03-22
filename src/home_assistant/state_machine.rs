use pyo3::prelude::*;

use crate::{
    home_assistant::entity_id::EntityId,
    python_utils::{detach, validate_type_by_name},
};

use super::state::State;

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
    pub fn get<Attributes: for<'py> FromPyObject<'py>, ContextEvent: for<'py> FromPyObject<'py>>(
        &self,
        py: Python<'_>,
        entity_id: EntityId,
    ) -> PyResult<Option<State<Attributes, ContextEvent>>> {
        let args = (entity_id.to_string(),);
        let state = self.0.call_method1(py, "get", args)?;
        state.extract(py)
    }
}
