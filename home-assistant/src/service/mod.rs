use service_domain::ServiceDomain;
use service_id::ServiceId;

pub mod service_domain;
pub mod service_id;

pub trait IntoServiceCall {
    type ServiceData;

    fn into_service_call(self) -> (ServiceDomain, ServiceId, Self::ServiceData);
}
