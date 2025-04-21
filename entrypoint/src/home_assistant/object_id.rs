use std::convert::Infallible;

use pyo3::{prelude::*, types::PyString};

use super::slug::Slug;

pub use super::slug::SlugParsingError as ObjectIdParsingError;

#[derive(Debug, Clone, derive_more::Display, derive_more::FromStr)]
pub struct ObjectId(pub Slug);

impl<'py> IntoPyObject<'py> for ObjectId {
    type Target = PyString;
    type Output = Bound<'py, Self::Target>;
    type Error = Infallible;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        let s = self.to_string();
        s.into_pyobject(py)
    }
}
