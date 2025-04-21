use chrono::DateTime;
use chrono_tz::Tz;
use ijson::{IArray, INumber, IObject, IString, IValue};
#[cfg(feature = "pyo3")]
use pyo3::{
    exceptions::{PyTypeError, PyValueError},
    prelude::*,
    types::{PyList, PyNone},
};
use snafu::Snafu;

use super::{finite_f64::FiniteF64, map::Map, map_key::MapKey};

#[derive(Debug, Clone, derive_more::From, derive_more::TryInto)]
pub enum Arbitrary {
    Null,
    Bool(bool),
    Integer(i64),
    Float(FiniteF64),
    String(String),
    Array(Vec<Arbitrary>),
    Map(Map),
    DateTime(DateTime<Tz>),
}

impl From<MapKey> for Arbitrary {
    fn from(map_key: MapKey) -> Self {
        match map_key {
            MapKey::Null => Arbitrary::Null,
            MapKey::Bool(b) => Arbitrary::Bool(b),
            MapKey::Integer(int) => Arbitrary::Integer(int),
            MapKey::String(s) => Arbitrary::String(s),
            // close enough
            MapKey::Tuple(vec) => Arbitrary::Array(vec.into_iter().map(Into::into).collect()),
            MapKey::DateTime(date_time) => Arbitrary::DateTime(date_time),
        }
    }
}

#[derive(Debug, Snafu)]
pub enum MapKeyFromArbitraryError {
    #[snafu(display("floats aren't supported as map keys yet. got {value:?}"))]
    FloatNotSupported { value: FiniteF64 },
    #[snafu(display("a map cannot be a map key. got {value:?}"))]
    MapCannotBeAMapKey { value: Map },
}

impl From<Arbitrary> for IValue {
    fn from(value: Arbitrary) -> Self {
        match value {
            Arbitrary::Null => IValue::NULL,
            Arbitrary::Bool(true) => IValue::TRUE,
            Arbitrary::Bool(false) => IValue::FALSE,
            Arbitrary::Integer(int) => INumber::from(int).into(),
            Arbitrary::Float(float) => INumber::try_from(f64::from(float)).unwrap().into(),
            Arbitrary::String(s) => IString::from(s).into(),
            Arbitrary::Array(vec) => {
                IArray::from_iter(vec.into_iter().map(Into::<IValue>::into)).into()
            }
            Arbitrary::Map(Map(btree_map)) => {
                let mut object = IObject::new();

                for (key, value) in btree_map {
                    let key: IString = key.into();
                    object.insert(key, value);
                }

                object.into()
            }
            Arbitrary::DateTime(date_time) => IString::from(date_time.to_rfc3339()).into(),
        }
    }
}

#[cfg(feature = "pyo3")]
impl<'py> FromPyObject<'py> for Arbitrary {
    fn extract_bound(ob: &Bound<'py, PyAny>) -> PyResult<Self> {
        if let Ok(map_key) = ob.extract::<MapKey>() {
            Ok(map_key.into())
        } else if let Ok(map) = ob.extract() {
            Ok(Self::Map(map))
        } else if let Ok(f) = ob.extract::<f64>() {
            let f = FiniteF64::try_from(f).map_err(|err| PyValueError::new_err(err.to_string()))?;
            Ok(Self::Float(f))
        } else if let Ok(vec) = ob.extract() {
            Ok(Self::Array(vec))
        } else {
            let type_name = ob.get_type().fully_qualified_name()?;
            Err(PyTypeError::new_err(format!(
                "can't extract an arbitrary from a {type_name}"
            )))
        }
    }
}

#[cfg(feature = "pyo3")]
impl<'py> IntoPyObject<'py> for Arbitrary {
    type Target = PyAny;

    type Output = Bound<'py, Self::Target>;

    type Error = PyErr;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        match self {
            Arbitrary::Null => Ok(PyNone::get(py).to_owned().into_any()),
            Arbitrary::Bool(b) => Ok(b.into_pyobject(py)?.to_owned().into_any()),
            Arbitrary::Integer(i) => Ok(i.into_pyobject(py)?.into_any()),
            Arbitrary::Float(finite_f64) => Ok(finite_f64.into_pyobject(py)?.into_any()),
            Arbitrary::String(s) => Ok(s.into_pyobject(py)?.into_any()),
            Arbitrary::Array(vec) => Ok(PyList::new(py, vec)?.into_any()),
            Arbitrary::Map(map) => Ok(map.into_pyobject(py)?.into_any()),
            Arbitrary::DateTime(date_time) => Ok(date_time.into_pyobject(py)?.into_any()),
        }
    }
}
