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
