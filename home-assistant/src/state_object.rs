use super::{
    event::{context::context::Context, specific::state_changed},
    home_assistant::HomeAssistant,
};
use crate::entity_id::EntityId;
use chrono::{DateTime, Utc};
use emitter_and_signal::signal::Signal;
use once_cell::sync::OnceCell;
use pyo3::{
    prelude::*,
    types::{PyCFunction, PyDict, PyTuple},
};
use std::{future::Future, sync::Arc};
use tokio::{select, sync::mpsc};

#[derive(Debug, FromPyObject)]
pub struct StateObject<State, Attributes, ContextEvent> {
    pub entity_id: EntityId,
    pub state: State,
    pub attributes: Attributes,
    pub last_changed: Option<DateTime<Utc>>,
    pub last_reported: Option<DateTime<Utc>>,
    pub last_updated: Option<DateTime<Utc>>,
    pub context: Context<ContextEvent>,
}

impl<
        State: Send + Sync + 'static + for<'py> FromPyObject<'py>,
        Attributes: Send + Sync + 'static + for<'py> FromPyObject<'py>,
        ContextEvent: Send + Sync + 'static + for<'py> FromPyObject<'py>,
    > StateObject<State, Attributes, ContextEvent>
{
    pub fn store(
        py: Python<'_>,
        home_assistant: &HomeAssistant,
        entity_id: EntityId,
    ) -> PyResult<(
        Signal<Option<Arc<Self>>>,
        impl Future<Output = Result<(), emitter_and_signal::signal::JoinError>>,
    )> {
        let state_machine = home_assistant.states(py)?;
        let current = state_machine.get(py, entity_id.clone())?;

        let py_home_assistant = home_assistant.into_pyobject(py)?.unbind();

        let (store, task) = Signal::new(current.map(Arc::new), |mut publisher_stream| async move {
            while let Some(publisher) = publisher_stream.wait().await {
                let (new_state_sender, mut new_state_receiver) = mpsc::channel(8);

                let untrack = Python::with_gil::<_, PyResult<_>>(|py| {
                    static EVENT_MODULE: OnceCell<Py<PyModule>> = OnceCell::new();

                    let event_module = EVENT_MODULE
                        .get_or_try_init(|| {
                            Result::<_, PyErr>::Ok(
                                py.import("homeassistant.helpers.event")?.unbind(),
                            )
                        })?
                        .bind(py);

                    let untrack = {
                        let callback =
                            move |args: &Bound<'_, PyTuple>,
                                  _kwargs: Option<&Bound<'_, PyDict>>| {
                                #[cfg(feature = "tracing")]
                                tracing::debug!("calling the closure");

                                if let Ok((event,)) = args.extract::<(
                                    state_changed::Event<
                                        State,
                                        Attributes,
                                        ContextEvent,
                                        State,
                                        Attributes,
                                        ContextEvent,
                                        Py<PyAny>,
                                    >,
                                )>() {
                                    let new_state = event.data.new_state;

                                    #[cfg(feature = "tracing")]
                                    tracing::debug!("sending a new state"); // TODO: remove
                                    new_state_sender.try_send(new_state).unwrap();
                                }
                            };
                        let callback = PyCFunction::new_closure(py, None, None, callback)?;
                        let args = (
                            py_home_assistant.clone_ref(py),
                            vec![entity_id.clone()],
                            callback,
                        );
                        event_module.call_method1("async_track_state_change_event", args)?
                    };
                    #[cfg(feature = "tracing")]
                    tracing::debug!(?untrack, "as any");

                    let is_callable = untrack.is_callable();
                    #[cfg(feature = "tracing")]
                    tracing::debug!(?is_callable);

                    // let untrack = untrack.downcast_into::<PyFunction>()?;
                    // tracing::debug!(?untrack, "as downcast");

                    let untrack = untrack.unbind();
                    #[cfg(feature = "tracing")]
                    tracing::debug!(?untrack, "as unbound");

                    Ok(untrack)
                });

                if let Ok(untrack) = untrack {
                    #[cfg(feature = "tracing")]
                    tracing::debug!("untrack is ok, going to wait for the next relevant event...");
                    loop {
                        select! {
                            biased;
                            _ = publisher.all_unsubscribed() => {
                                #[cfg(feature = "tracing")]
                                tracing::debug!("calling untrack");
                                let res = Python::with_gil(|py| untrack.call0(py));

                                #[cfg(feature = "tracing")]
                                tracing::debug!(?res);
                                break;
                            }
                            new_state = new_state_receiver.recv() => {
                                match new_state {
                                    Some(new_state) => {
                                        #[cfg(feature = "tracing")]
                                        tracing::debug!("publishing new state");
                                        publisher.publish(new_state.map(Arc::new))
                                    },
                                    None => {
                                        #[cfg(feature = "tracing")]
                                        tracing::debug!("channel dropped");
                                        break
                                    },
                                }
                            }
                        }
                    }
                } else {
                    #[cfg(feature = "tracing")]
                    tracing::debug!("untrack is err");
                }
            }
        });

        Ok((store, task))
    }
}
