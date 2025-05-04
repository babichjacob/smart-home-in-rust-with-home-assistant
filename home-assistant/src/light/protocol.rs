use super::service::{turn_off::TurnOff, turn_on::TurnOn};
use super::{state::LightState, GetStateObjectError, HomeAssistantLight};
use crate::{
    event::context::context::Context,
    state::{ErrorState, HomeAssistantState, UnexpectedState},
};
use protocol::light::{GetState, SetState};
use pyo3::prelude::*;
use python_utils::IsNone;
use snafu::{ResultExt, Snafu};

#[derive(Debug, Snafu)]
pub enum GetStateError {
    GetStateObjectError { source: GetStateObjectError },
    Error { state: ErrorState },
    UnexpectedError { state: UnexpectedState },
}

impl GetState for HomeAssistantLight {
    type Error = GetStateError;

    async fn get_state(&self) -> Result<protocol::light::State, Self::Error> {
        let state_object = self.get_state_object().context(GetStateObjectSnafu)?;
        let state = state_object.state;

        match state {
            HomeAssistantState::Ok(light_state) => Ok(light_state.into()),
            HomeAssistantState::Err(error_state) => {
                Err(GetStateError::Error { state: error_state })
            }
            HomeAssistantState::UnexpectedErr(state) => {
                Err(GetStateError::UnexpectedError { state })
            }
        }
    }
}

impl SetState for HomeAssistantLight {
    type Error = PyErr;

    async fn set_state(&mut self, state: protocol::light::State) -> Result<(), Self::Error> {
        let context: Option<Context<()>> = None;
        let target: Option<()> = None;

        let services = Python::with_gil(|py| self.home_assistant.services(py))?;

        let _: IsNone = match state {
            protocol::light::State::Off => {
                services
                    .call_service(
                        TurnOff {
                            entity_id: self.entity_id(),
                        },
                        context,
                        target,
                        false,
                    )
                    .await
            }
            protocol::light::State::On => {
                services
                    .call_service(
                        TurnOn {
                            entity_id: self.entity_id(),
                        },
                        context,
                        target,
                        false,
                    )
                    .await
            }
        }?;

        Ok(())
    }
}
