use chrono::{DateTime, Utc};
use pyo3::prelude::*;

use super::{context::context::Context, event_origin::EventOrigin};

/// Representation of an event within the bus.
#[derive(Debug, FromPyObject)]
pub struct Event<Type, Data> {
    pub event_type: Type,
    pub data: Data,
    pub origin: EventOrigin,
    /// In order to prevent cycles, the user must extract this to a [`Context`](super::context::Context) themself, using the [`context`](Self::context) method
    context: Py<PyAny>,
    time_fired_timestamp: f64,
}

impl<Type, Data> Event<Type, Data> {
    pub fn context<'py>(&self, py: Python<'py>) -> PyResult<Context> {
        self.context.extract(py)
    }

    pub fn time_fired(&self) -> Option<DateTime<Utc>> {
        const NANOS_PER_SEC: i32 = 1_000_000_000;

        let secs = self.time_fired_timestamp as i64;
        let nsecs = (self.time_fired_timestamp.fract() * (NANOS_PER_SEC as f64)) as u32;

        DateTime::from_timestamp(secs, nsecs)
    }
}
