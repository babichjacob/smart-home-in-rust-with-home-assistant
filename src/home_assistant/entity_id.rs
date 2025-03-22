use std::{convert::Infallible, fmt::Display, str::FromStr};

use pyo3::{exceptions::PyValueError, prelude::*, types::PyString};
use snafu::{ResultExt, Snafu};

use super::{
    domain::Domain,
    object_id::{ObjectId, ObjectIdParsingError},
};

#[derive(Debug, Clone)]
pub struct EntityId(pub Domain, pub ObjectId);

#[derive(Debug, Clone, Snafu)]
pub enum EntityIdParsingError {
    #[snafu(display("entity IDs have a dot / period in them, e.g. light.kitchen_lamp"))]
    MissingDot,

    #[snafu(display("could not parse the domain part of the entity ID"))]
    ParsingDomain { source: <Domain as FromStr>::Err },

    #[snafu(display("could not parse the object ID part of the entity ID"))]
    ParsingObjectId { source: ObjectIdParsingError },
}

impl FromStr for EntityId {
    type Err = EntityIdParsingError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (domain, object_id) = s.split_once('.').ok_or(EntityIdParsingError::MissingDot)?;

        let domain = domain.parse().context(ParsingDomainSnafu)?;
        let object_id = object_id.parse().context(ParsingObjectIdSnafu)?;

        Ok(Self(domain, object_id))
    }
}

impl Display for EntityId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Self(domain, object_id) = self;

        write!(f, "{domain}.{object_id}")
    }
}

impl From<EntityIdParsingError> for PyErr {
    fn from(error: EntityIdParsingError) -> Self {
        PyValueError::new_err(error.to_string())
    }
}

impl<'py> FromPyObject<'py> for EntityId {
    fn extract_bound(ob: &Bound<'py, PyAny>) -> PyResult<Self> {
        let s = ob.extract()?;
        let entity_id = EntityId::from_str(s)?;

        Ok(entity_id)
    }
}

impl<'py> IntoPyObject<'py> for &EntityId {
    type Target = PyString;
    type Output = Bound<'py, Self::Target>;
    type Error = Infallible;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        let s = self.to_string();
        s.into_pyobject(py)
    }
}
