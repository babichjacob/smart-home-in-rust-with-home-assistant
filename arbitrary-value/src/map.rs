use std::collections::BTreeMap;

use super::{arbitrary::Arbitrary, map_key::MapKey};

#[cfg_attr(feature = "pyo3", derive(pyo3::FromPyObject, pyo3::IntoPyObject))]
#[derive(Debug, Clone, Default, derive_more::From, derive_more::Into)]
pub struct Map(pub BTreeMap<MapKey, Arbitrary>);
