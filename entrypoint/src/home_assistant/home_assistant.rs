use std::convert::Infallible;

use pyo3::prelude::*;

use crate::python_utils::{detach, validate_type_by_name};

use super::{service_registry::ServiceRegistry, state_machine::StateMachine};

#[derive(Debug)]
pub struct HomeAssistant(Py<PyAny>);

impl<'source> FromPyObject<'source> for HomeAssistant {
    fn extract_bound(ob: &Bound<'source, PyAny>) -> PyResult<Self> {
        // region: Validation
        validate_type_by_name(ob, "HomeAssistant")?;
        // endregion: Validation

        Ok(Self(detach(ob)))
    }
}

impl<'py> IntoPyObject<'py> for &HomeAssistant {
    type Target = PyAny;
    type Output = Bound<'py, Self::Target>;
    type Error = Infallible;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        Ok(self.0.bind(py).to_owned())
    }
}

impl HomeAssistant {
    /// Return the representation
    pub fn repr(&self, py: Python<'_>) -> Result<String, PyErr> {
        let bound = self.0.bind(py);
        let repr = bound.repr()?;
        repr.extract()
    }

    /// Return if Home Assistant is running.
    pub fn is_running(&self, py: Python<'_>) -> Result<bool, PyErr> {
        let is_running = self.0.getattr(py, "is_running")?;
        is_running.extract(py)
    }
    /// Return if Home Assistant is stopping.
    pub fn is_stopping(&self, py: Python<'_>) -> Result<bool, PyErr> {
        let is_stopping = self.0.getattr(py, "is_stopping")?;
        is_stopping.extract(py)
    }

    pub fn states(&self, py: Python<'_>) -> Result<StateMachine, PyErr> {
        let states = self.0.getattr(py, "states")?;
        states.extract(py)
    }

    pub fn services(&self, py: Python<'_>) -> Result<ServiceRegistry, PyErr> {
        let services = self.0.getattr(py, "services")?;
        services.extract(py)
    }
}
