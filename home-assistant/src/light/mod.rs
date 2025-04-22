use attributes::LightAttributes;
use pyo3::prelude::*;
use snafu::{ResultExt, Snafu};
use state::LightState;

use crate::state::HomeAssistantState;

use super::{
    domain::Domain, entity_id::EntityId, home_assistant::HomeAssistant, object_id::ObjectId,
    state_object::StateObject,
};

mod attributes;
mod protocol;
mod service;
mod state;

#[derive(Debug)]
pub struct HomeAssistantLight {
    pub home_assistant: HomeAssistant,
    pub object_id: ObjectId,
}

impl HomeAssistantLight {
    fn entity_id(&self) -> EntityId {
        EntityId(Domain::Light, self.object_id.clone())
    }
}

#[derive(Debug, Snafu)]
pub enum GetStateObjectError {
    PythonError { source: PyErr },
    EntityMissing,
}

impl HomeAssistantLight {
    fn get_state_object(
        &self,
    ) -> Result<
        StateObject<HomeAssistantState<LightState>, LightAttributes, Py<PyAny>>,
        GetStateObjectError,
    > {
        Python::with_gil(|py| {
            let states = self.home_assistant.states(py).context(PythonSnafu)?;
            let entity_id = self.entity_id();
            let state_object = states
                .get(py, entity_id)
                .context(PythonSnafu)?
                .ok_or(GetStateObjectError::EntityMissing)?;

            Ok(state_object)
        })
    }
}
