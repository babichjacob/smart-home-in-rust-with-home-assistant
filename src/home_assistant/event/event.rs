use chrono::{DateTime, Utc};
use pyo3::prelude::*;

use super::event_origin::EventOrigin;

/// Representation of an event within the bus.
#[derive(Debug, FromPyObject)]
pub struct Event<Type, Data, Context> {
    pub event_type: Type,
    pub data: Data,
    pub origin: EventOrigin,
    /// In order to prevent cycles, the user must decide to pass [`Py<PyAny>`] for the `Context` type here
    /// or for the `Event` type in [`Context`]
    pub context: Context,
    time_fired_timestamp: f64,
}

impl<Type, Data, Context> Event<Type, Data, Context> {
    pub fn time_fired(&self) -> Option<DateTime<Utc>> {
        const NANOS_PER_SEC: i32 = 1_000_000_000;

        let secs = self.time_fired_timestamp as i64;
        let nsecs = (self.time_fired_timestamp.fract() * (NANOS_PER_SEC as f64)) as u32;

        DateTime::from_timestamp(secs, nsecs)
    }
}
