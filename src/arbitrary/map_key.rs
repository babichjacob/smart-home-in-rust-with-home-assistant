use std::fmt::Display;

use chrono::DateTime;
use chrono_tz::Tz;
use ijson::IString;
use itertools::Itertools;
use pyo3::{
    exceptions::PyTypeError,
    prelude::*,
    types::{PyNone, PyTuple},
};

use super::arbitrary::{Arbitrary, MapKeyFromArbitraryError};

#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum MapKey {
    Null,
    Bool(bool),
    Integer(i64),
    String(String),
    Tuple(Vec<MapKey>),
    DateTime(DateTime<Tz>),
}

impl Display for MapKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MapKey::Null => write!(f, "null"),
            MapKey::Bool(b) => write!(f, "{b}"),
            MapKey::Integer(i) => write!(f, "{i}"),
            MapKey::String(s) => write!(f, "{s}"),
            MapKey::Tuple(vec) => {
                let comma_separated =
                    Itertools::intersperse(vec.iter().map(ToString::to_string), ", ".to_string());
                write!(f, "({})", String::from_iter(comma_separated))
            }
            MapKey::DateTime(date_time) => {
                write!(f, "{}", date_time.to_rfc3339())
            }
        }
    }
}

impl<'py> FromPyObject<'py> for MapKey {
    fn extract_bound(ob: &Bound<'py, PyAny>) -> PyResult<Self> {
        if let Ok(_none) = ob.downcast::<PyNone>() {
            Ok(Self::Null)
        } else if let Ok(b) = ob.extract() {
            Ok(Self::Bool(b))
        } else if let Ok(int) = ob.extract() {
            Ok(Self::Integer(int))
        } else if let Ok(s) = ob.extract() {
            Ok(Self::String(s))
        } else if let Ok(tuple) = ob.extract() {
            Ok(Self::Tuple(tuple))
        } else {
            let type_name = ob.get_type().fully_qualified_name()?;
            Err(PyTypeError::new_err(format!(
                "can't extract a map key from a {type_name}"
            )))
        }
    }
}

impl<'py> IntoPyObject<'py> for MapKey {
    type Target = PyAny;

    type Output = Bound<'py, Self::Target>;

    type Error = PyErr;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        match self {
            MapKey::Null => Ok(PyNone::get(py).to_owned().into_any()),
            MapKey::Bool(b) => Ok(b.into_pyobject(py)?.to_owned().into_any()),
            MapKey::Integer(i) => Ok(i.into_pyobject(py)?.into_any()),
            MapKey::String(s) => Ok(s.into_pyobject(py)?.into_any()),
            MapKey::Tuple(vec) => Ok(PyTuple::new(py, vec)?.into_any()),
            MapKey::DateTime(date_time) => Ok(date_time.into_pyobject(py)?.into_any()),
        }
    }
}

impl From<MapKey> for IString {
    fn from(map_key: MapKey) -> Self {
        Self::from(map_key.to_string())
    }
}

impl TryFrom<Arbitrary> for MapKey {
    type Error = MapKeyFromArbitraryError;

    fn try_from(arbitrary: Arbitrary) -> Result<Self, Self::Error> {
        match arbitrary {
            Arbitrary::Null => Ok(MapKey::Null),
            Arbitrary::Bool(b) => Ok(MapKey::Bool(b)),
            Arbitrary::Integer(i) => Ok(MapKey::Integer(i)),
            Arbitrary::String(s) => Ok(MapKey::String(s)),
            Arbitrary::Array(vec) => {
                let tuple = Result::from_iter(vec.into_iter().map(TryInto::try_into))?;
                Ok(MapKey::Tuple(tuple))
            }
            Arbitrary::DateTime(date_time) => Ok(MapKey::DateTime(date_time)),
            Arbitrary::Float(float) => {
                Err(MapKeyFromArbitraryError::FloatNotSupported { value: float })
            }
            Arbitrary::Map(map) => Err(MapKeyFromArbitraryError::MapCannotBeAMapKey { value: map }),
        }
    }
}
