use std::convert::Infallible;

use pyo3::{exceptions::PyTypeError, prelude::*, types::PyNone};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct IsNone;

impl<'py> FromPyObject<'py> for IsNone {
    fn extract_bound(ob: &Bound<'py, PyAny>) -> PyResult<Self> {
        ob.downcast::<PyNone>()?;
        Ok(IsNone)
    }
}

impl<'py> IntoPyObject<'py> for IsNone {
    type Target = PyNone;

    type Output = Borrowed<'py, 'py, Self::Target>;

    type Error = Infallible;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        Ok(PyNone::get(py))
    }
}

/// Create a GIL-independent reference
pub fn detach<T>(bound: &Bound<T>) -> Py<T> {
    let py = bound.py();
    bound.as_unbound().clone_ref(py)
}

pub fn validate_type_by_name(bound: &Bound<PyAny>, expected_type_name: &str) -> PyResult<()> {
    let py_type = bound.get_type();
    let type_name = py_type.name()?;
    let type_name = type_name.to_str()?;

    if type_name != expected_type_name {
        let fully_qualified_type_name = py_type.fully_qualified_name()?;
        let fully_qualified_type_name = fully_qualified_type_name.to_str()?;
        return Err(PyTypeError::new_err(format!("expected an instance of {expected_type_name} but got an instance of {fully_qualified_type_name}")));
    }

    return Ok(());
}
