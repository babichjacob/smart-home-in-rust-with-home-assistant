use std::convert::Infallible;

use pyo3::{prelude::*, types::PyString};
use smol_str::SmolStr;
use ulid::Ulid;

#[derive(Debug, Clone)]
pub enum Id {
    Ulid(Ulid),
    Other(SmolStr),
}

impl<'py> FromPyObject<'py> for Id {
    fn extract_bound(ob: &Bound<'py, PyAny>) -> PyResult<Self> {
        let s = ob.extract::<String>()?;

        if let Ok(ulid) = s.parse() {
            Ok(Id::Ulid(ulid))
        } else {
            Ok(Id::Other(s.into()))
        }
    }
}

impl<'py> IntoPyObject<'py> for Id {
    type Target = PyString;

    type Output = Bound<'py, Self::Target>;

    type Error = Infallible;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        match self {
            Id::Ulid(ulid) => ulid.to_string().into_pyobject(py),
            Id::Other(id) => id.as_str().into_pyobject(py),
        }
    }
}
