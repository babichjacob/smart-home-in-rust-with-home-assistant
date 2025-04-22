use super::id::Id;
use once_cell::sync::OnceCell;
use pyo3::{prelude::*, types::PyType};

/// The context that triggered something.
#[derive(Debug, FromPyObject)]
pub struct Context<Event> {
    pub id: Id,
    pub user_id: Option<String>,
    pub parent_id: Option<String>,
    /// In order to prevent cycles, the user must decide to pass [`Py<PyAny>`] for the `Event` type here
    /// or for the `Context` type in [`Event`]
    pub origin_event: Event,
}

impl<'py, Event: IntoPyObject<'py>> IntoPyObject<'py> for Context<Event> {
    type Target = PyAny;

    type Output = Bound<'py, Self::Target>;

    type Error = PyErr;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        static HOMEASSISTANT_CORE: OnceCell<Py<PyModule>> = OnceCell::new();

        let homeassistant_core = HOMEASSISTANT_CORE
            .get_or_try_init(|| Result::<_, PyErr>::Ok(py.import("homeassistant.core")?.unbind()))?
            .bind(py);

        let context_class = homeassistant_core.getattr("Context")?;
        let context_class = context_class.downcast_into::<PyType>()?;

        let context_instance = context_class.call1((self.user_id, self.parent_id, self.id))?;

        context_instance.setattr("origin_event", self.origin_event)?;

        Ok(context_instance)
    }
}
