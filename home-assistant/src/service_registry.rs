use super::{event::context::context::Context, service::IntoServiceCall};
use pyo3::prelude::*;
use python_utils::{detach, validate_type_by_name};

#[derive(Debug)]
pub struct ServiceRegistry(Py<PyAny>);

impl<'py> FromPyObject<'py> for ServiceRegistry {
    fn extract_bound(ob: &Bound<'py, PyAny>) -> PyResult<Self> {
        // region: Validation
        validate_type_by_name(ob, "ServiceRegistry")?;
        // endregion: Validation

        Ok(Self(detach(ob)))
    }
}

impl ServiceRegistry {
    pub async fn call_service<
        ServiceData: for<'py> IntoPyObject<'py>,
        Target: for<'py> IntoPyObject<'py>,
        Event: for<'py> IntoPyObject<'py>,
        ServiceResponse: for<'py> FromPyObject<'py>,
    >(
        &self,
        service_call: impl IntoServiceCall<ServiceData = ServiceData>,
        context: Option<Context<Event>>,
        target: Option<Target>,
        return_response: bool,
    ) -> PyResult<ServiceResponse> {
        let (domain, service, service_data) = service_call.into_service_call();

        let blocking = true;

        let args = (
            domain,
            service,
            service_data,
            blocking,
            context,
            target,
            return_response,
        );

        let future = Python::with_gil::<_, PyResult<_>>(|py| {
            let service_registry = self.0.bind(py);
            let awaitable = service_registry.call_method("async_call", args, None)?;
            pyo3_async_runtimes::tokio::into_future(awaitable)
        })?;

        let service_response = future.await?;
        Python::with_gil(|py| service_response.extract(py))
    }
}
