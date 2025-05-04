use std::str::FromStr;

use pyo3::{exceptions::PyValueError, prelude::*};
use strum::EnumString;

#[derive(Debug, Clone, EnumString, strum::Display)]
#[strum(serialize_all = "snake_case")]
pub enum LightState {
    On,
    Off,
}

impl<'py> FromPyObject<'py> for LightState {
    fn extract_bound(ob: &Bound<'py, PyAny>) -> PyResult<Self> {
        let s = ob.extract::<String>()?;

        let state =
            LightState::from_str(&s).map_err(|err| PyValueError::new_err(err.to_string()))?;

        Ok(state)
    }
}

impl From<LightState> for protocol::light::State {
    fn from(light_state: LightState) -> Self {
        match light_state {
            LightState::On => protocol::light::State::On,
            LightState::Off => protocol::light::State::Off,
        }
    }
}

impl From<protocol::light::State> for LightState {
    fn from(state: protocol::light::State) -> Self {
        match state {
            protocol::light::State::On => LightState::On,
            protocol::light::State::Off => LightState::Off,
        }
    }
}
