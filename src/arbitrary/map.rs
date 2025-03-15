use std::collections::BTreeMap;

use pyo3::prelude::*;

use super::{arbitrary::Arbitrary, map_key::MapKey};

#[derive(Debug, Clone, Default, derive_more::From, derive_more::Into, IntoPyObject)]
pub struct Map(pub BTreeMap<MapKey, Arbitrary>);

impl<'py> FromPyObject<'py> for Map {
    fn extract_bound(ob: &Bound<'py, PyAny>) -> PyResult<Self> {
        let inner = ob.extract()?;

        Ok(Self(inner))
    }
}
