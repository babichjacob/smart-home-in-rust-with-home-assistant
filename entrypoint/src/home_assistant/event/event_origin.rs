use std::str::FromStr;

use pyo3::{exceptions::PyValueError, prelude::*};

#[derive(Debug, Clone, strum::EnumString, strum::Display)]
#[strum(serialize_all = "UPPERCASE")]
pub enum EventOrigin {
    Local,
    Remote,
}

impl<'py> FromPyObject<'py> for EventOrigin {
    fn extract_bound(ob: &Bound<'py, PyAny>) -> PyResult<Self> {
        let s = ob.str()?;
        let s = s.extract()?;
        let event_origin =
            EventOrigin::from_str(s).map_err(|err| PyValueError::new_err(err.to_string()))?;

        Ok(event_origin)
    }
}
