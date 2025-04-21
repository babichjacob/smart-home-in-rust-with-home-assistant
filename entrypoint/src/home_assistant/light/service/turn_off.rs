use std::str::FromStr;

use pyo3::IntoPyObject;

use crate::home_assistant::{
    entity_id::EntityId,
    service::{service_domain::ServiceDomain, service_id::ServiceId, IntoServiceCall},
};

#[derive(Debug, Clone)]
pub struct TurnOff {
    pub entity_id: EntityId,
}

#[derive(Debug, Clone, IntoPyObject)]
pub struct TurnOffServiceData {
    entity_id: EntityId,
}

impl IntoServiceCall for TurnOff {
    type ServiceData = TurnOffServiceData;

    fn into_service_call(self) -> (ServiceDomain, ServiceId, Self::ServiceData) {
        let service_domain = ServiceDomain::from_str("light").expect("statically written and known to be a valid slug; hoping to get compiler checks instead in the future");
        let service_id = ServiceId::from_str("turn_off").expect("statically written and known to be a valid slug; hoping to get compiler checks instead in the future");

        let Self { entity_id } = self;

        let service_data = TurnOffServiceData { entity_id };

        (service_domain, service_id, service_data)
    }
}
