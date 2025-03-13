use std::collections::BTreeMap;

use pyo3::prelude::*;

use super::{arbitrary::Arbitrary, map_key::MapKey};

#[derive(Debug, Clone, derive_more::From, derive_more::Into)]
pub struct Map(pub BTreeMap<MapKey, Arbitrary>);

impl<'py> FromPyObject<'py> for Map {
    fn extract_bound(ob: &Bound<'py, PyAny>) -> PyResult<Self> {
        let inner: BTreeMap<MapKey, Arbitrary> = ob.extract()?;

        Ok(Self(inner))
    }
}
