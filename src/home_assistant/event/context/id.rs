use pyo3::prelude::*;
use ulid::Ulid;

#[derive(Debug, Clone)]
pub enum Id {
    Ulid(Ulid),
    Other(String),
}

impl<'py> FromPyObject<'py> for Id {
    fn extract_bound(ob: &Bound<'py, PyAny>) -> PyResult<Self> {
        let s = ob.extract::<String>()?;

        if let Ok(ulid) = s.parse() {
            Ok(Id::Ulid(ulid))
        } else {
            Ok(Id::Other(s))
        }
    }
}
