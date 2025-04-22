use std::str::FromStr;

use pyo3::IntoPyObject;

use crate::{
    entity_id::EntityId,
    service::{service_domain::ServiceDomain, service_id::ServiceId, IntoServiceCall},
};

#[derive(Debug, Clone)]
pub struct TurnOn {
    pub entity_id: EntityId,
}

#[derive(Debug, Clone, IntoPyObject)]
pub struct TurnOnServiceData {
    entity_id: EntityId,
}

impl IntoServiceCall for TurnOn {
    type ServiceData = TurnOnServiceData;

    fn into_service_call(self) -> (ServiceDomain, ServiceId, Self::ServiceData) {
        let service_domain = ServiceDomain::from_str("light").expect("statically written and known to be a valid slug; hoping to get compiler checks instead in the future");
        let service_id = ServiceId::from_str("turn_on").expect("statically written and known to be a valid slug; hoping to get compiler checks instead in the future");

        let Self { entity_id } = self;
        let service_data = TurnOnServiceData { entity_id };

        (service_domain, service_id, service_data)
    }
}
