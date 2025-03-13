use std::{str::FromStr, sync::Arc};

use pyo3::{exceptions::PyValueError, PyErr};
use snafu::Snafu;

#[derive(Debug, Clone, derive_more::Display)]
pub struct ObjectId(Arc<str>);

#[derive(Debug, Clone, Snafu)]
#[snafu(display("expected a lowercase ASCII alphabetical character (i.e. a through z) or an underscore (i.e. _) but encountered {encountered}"))]
pub struct ObjectIdParsingError {
    encountered: char,
}

impl FromStr for ObjectId {
    type Err = ObjectIdParsingError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        for c in s.chars() {
            match c {
                'a'..='z' => {}
                '_' => {}
                _ => return Err(ObjectIdParsingError { encountered: c }),
            }
        }

        Ok(Self(s.into()))
    }
}

impl From<ObjectIdParsingError> for PyErr {
    fn from(error: ObjectIdParsingError) -> Self {
        PyValueError::new_err(error.to_string())
    }
}
