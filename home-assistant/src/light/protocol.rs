use super::service::{turn_off::TurnOff, turn_on::TurnOn};
use super::{state::LightState, GetStateObjectError, HomeAssistantLight};
use crate::{
    event::context::context::Context,
    state::{ErrorState, HomeAssistantState, UnexpectedState},
};
use arbitrary_value::arbitrary::Arbitrary;
use protocol::light::Light;
use pyo3::prelude::*;
use snafu::{ResultExt, Snafu};

#[derive(Debug, Snafu)]
pub enum IsStateError {
    GetStateObjectError { source: GetStateObjectError },
    Error { state: ErrorState },
    UnexpectedError { state: UnexpectedState },
}

impl Light for HomeAssistantLight {
    type IsOnError = IsStateError;

    async fn is_on(&self) -> Result<bool, Self::IsOnError> {
        let state_object = self.get_state_object().context(GetStateObjectSnafu)?;
        let state = state_object.state;

        match state {
            HomeAssistantState::Ok(light_state) => Ok(matches!(light_state, LightState::On)),
            HomeAssistantState::Err(state) => Err(IsStateError::Error { state }),
            HomeAssistantState::UnexpectedErr(state) => {
                Err(IsStateError::UnexpectedError { state })
            }
        }
    }

    type IsOffError = IsStateError;

    async fn is_off(&self) -> Result<bool, Self::IsOffError> {
        let state_object = self.get_state_object().context(GetStateObjectSnafu)?;
        let state = state_object.state;

        match state {
            HomeAssistantState::Ok(light_state) => Ok(matches!(light_state, LightState::Off)),
            HomeAssistantState::Err(state) => Err(IsStateError::Error { state }),
            HomeAssistantState::UnexpectedErr(state) => {
                Err(IsStateError::UnexpectedError { state })
            }
        }
    }

    type TurnOnError = PyErr;

    async fn turn_on(&mut self) -> Result<(), Self::TurnOnError> {
        let context: Option<Context<()>> = None;
        let target: Option<()> = None;

        let services = Python::with_gil(|py| self.home_assistant.services(py))?;
        // TODO
        let service_response: Arbitrary = services
            .call_service(
                TurnOn {
                    entity_id: self.entity_id(),
                },
                context,
                target,
                false,
            )
            .await?;

        // TODO
        #[cfg(feature = "tracing")]
        tracing::info!(?service_response);

        Ok(())
    }

    type TurnOffError = PyErr;

    async fn turn_off(&mut self) -> Result<(), Self::TurnOffError> {
        let context: Option<Context<()>> = None;
        let target: Option<()> = None;

        let services = Python::with_gil(|py| self.home_assistant.services(py))?;
        // TODO
        let service_response: Arbitrary // TODO: a type that validates as None
         = services
            .call_service(
                TurnOff {
                    entity_id: self.entity_id(),
                },
                context,
                target,
                false,
            )
            .await?;

        // TODO
        #[cfg(feature = "tracing")]
        tracing::info!(?service_response);

        Ok(())
    }

    type ToggleError = PyErr;

    async fn toggle(&mut self) -> Result<(), Self::ToggleError> {
        todo!()
    }
}
