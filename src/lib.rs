use std::time::Duration;

use home_assistant::home_assistant::HomeAssistant;
use pyo3::prelude::*;
use shadow_rs::shadow;
use tokio::time::interval;
use tracing::Level;
use tracing_subscriber::fmt::format::FmtSpan;

mod arbitrary;
mod home_assistant;
mod python_utils;

shadow!(build_info);

async fn real_main(home_assistant: HomeAssistant) -> ! {
    tracing_subscriber::fmt()
        .with_max_level(Level::TRACE)
        .with_span_events(FmtSpan::ACTIVE)
        .pretty()
        .init();

    let built_at = build_info::BUILD_TIME;
    tracing::info!(built_at);

    let duration = Duration::from_millis(5900);
    let mut interval = interval(duration);

    loop {
        let instant = interval.tick().await;

        tracing::debug!(?instant, "it is now");
    }
}

#[pyfunction]
fn main<'p>(py: Python<'p>, home_assistant: HomeAssistant) -> PyResult<Bound<'p, PyAny>> {
    pyo3_async_runtimes::tokio::future_into_py::<_, ()>(py, async {
        real_main(home_assistant).await;
    })
}

/// A Python module implemented in Rust.
#[pymodule]
fn smart_home_in_rust_with_home_assistant(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(main, m)?)?;
    Ok(())
}
