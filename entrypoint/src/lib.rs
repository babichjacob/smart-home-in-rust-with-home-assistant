use std::{str::FromStr, time::Duration};

use driver_kasa::connection::LB130USHandle;
use home_assistant::{
    home_assistant::HomeAssistant, light::HomeAssistantLight, object_id::ObjectId,
};
use protocol::light::Light;
use pyo3::prelude::*;
use shadow_rs::shadow;
use tokio::time::interval;
use tracing::{level_filters::LevelFilter, Level};
use tracing_subscriber::{
    fmt::{self, format::FmtSpan},
    layer::SubscriberExt,
    registry,
    util::SubscriberInitExt,
    Layer,
};
use tracing_to_home_assistant::TracingToHomeAssistant;

mod home_assistant;
mod python_utils;
mod tracing_to_home_assistant;

shadow!(build_info);

async fn real_main(home_assistant: HomeAssistant) -> ! {
    registry()
        .with(
            fmt::layer()
                .pretty()
                .with_span_events(FmtSpan::ACTIVE)
                .with_filter(LevelFilter::from_level(Level::TRACE)),
        )
        .with(TracingToHomeAssistant)
        .init();

    let built_at = build_info::BUILD_TIME;
    tracing::info!(built_at);

    // let lamp = HomeAssistantLight {
    //     home_assistant,
    //     object_id: ObjectId::from_str("jacob_s_lamp_top").unwrap(),
    // };

    let ip = [10, 0, 3, 71];
    let port = 9999;

    let some_light = LB130USHandle::new(
        (ip, port).into(),
        Duration::from_secs(10),
        (64).try_into().unwrap(),
    );

    let mut interval = interval(Duration::from_secs(20));
    interval.tick().await;
    loop {
        interval.tick().await;

        tracing::info!("about to call get_sysinfo");
        let sysinfo_res = some_light.get_sysinfo().await;
        tracing::info!(?sysinfo_res, "got sys info");

        // let is_on = lamp.is_on().await;
        // tracing::info!(?is_on);
        // let is_off = lamp.is_off().await;
        // tracing::info!(?is_off);

        // let something = lamp.turn_on().await;
        // tracing::info!(?something);
    }
}

#[pyfunction]
fn main<'py>(py: Python<'py>, home_assistant: HomeAssistant) -> PyResult<Bound<'py, PyAny>> {
    pyo3_async_runtimes::tokio::future_into_py::<_, ()>(py, async {
        real_main(home_assistant).await;
    })
}

/// A Python module implemented in Rust.
#[pymodule]
fn smart_home_in_rust_with_home_assistant(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_function(wrap_pyfunction!(main, module)?)?;
    Ok(())
}
