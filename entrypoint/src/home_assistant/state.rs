use std::{convert::Infallible, str::FromStr};

use pyo3::{exceptions::PyValueError, prelude::*};
use smol_str::SmolStr;
use strum::EnumString;

/// A state in Home Assistant that is known to represent an error of some kind:
/// * `unavailable` (the device is likely offline or unreachable from the Home Assistant instance)
/// * `unknown` (I don't know how to explain this one)
#[derive(Debug, Clone, EnumString, strum::Display)]
#[strum(serialize_all = "snake_case")]
pub enum ErrorState {
    Unavailable,
    Unknown,
}

impl<'py> FromPyObject<'py> for ErrorState {
    fn extract_bound(ob: &Bound<'py, PyAny>) -> PyResult<Self> {
        let s = ob.extract::<String>()?;

        let state =
            ErrorState::from_str(&s).map_err(|err| PyValueError::new_err(err.to_string()))?;

        Ok(state)
    }
}

#[derive(Debug, Clone, derive_more::Display, derive_more::FromStr)]
pub struct UnexpectedState(pub SmolStr);

impl<'py> FromPyObject<'py> for UnexpectedState {
    fn extract_bound(ob: &Bound<'py, PyAny>) -> PyResult<Self> {
        let s = ob.extract::<String>()?;
        let s = SmolStr::new(s);

        Ok(UnexpectedState(s))
    }
}

#[derive(Debug, Clone, derive_more::Display)]
pub enum HomeAssistantState<State> {
    Ok(State),
    Err(ErrorState),
    UnexpectedErr(UnexpectedState),
}

impl<State: FromStr> FromStr for HomeAssistantState<State> {
    type Err = Infallible;

    fn from_str(s: &str) -> Result<Self, <Self as FromStr>::Err> {
        if let Ok(ok) = State::from_str(s) {
            return Ok(HomeAssistantState::Ok(ok));
        }

        if let Ok(error) = ErrorState::from_str(s) {
            return Ok(HomeAssistantState::Err(error));
        }

        Ok(HomeAssistantState::UnexpectedErr(UnexpectedState(s.into())))
    }
}

impl<'py, State: FromStr + FromPyObject<'py>> FromPyObject<'py> for HomeAssistantState<State> {
    fn extract_bound(ob: &Bound<'py, PyAny>) -> PyResult<Self> {
        let s = ob.extract::<String>()?;

        let Ok(state) = s.parse();

        Ok(state)
    }
}
