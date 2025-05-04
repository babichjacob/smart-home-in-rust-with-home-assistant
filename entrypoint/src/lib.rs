use std::{path::PathBuf, str::FromStr, time::Duration};

use clap::Parser;
use driver_kasa::connection::LB130USHandle;
use home_assistant::{
    home_assistant::HomeAssistant, light::HomeAssistantLight, object_id::ObjectId,
};
use protocol::light::{IsOff, IsOn};
use pyo3::prelude::*;
use shadow_rs::shadow;
use tokio::time::interval;
use tracing::{level_filters::LevelFilter, Level};
use tracing_appender::rolling::{self, RollingFileAppender};
use tracing_subscriber::{
    fmt::{self, fmt, format::FmtSpan},
    layer::SubscriberExt,
    registry,
    util::SubscriberInitExt,
    Layer,
};
use tracing_to_home_assistant::TracingToHomeAssistant;

mod tracing_to_home_assistant;

shadow!(build_info);

#[derive(Debug, Parser)]
struct Args {
    #[arg(env)]
    persistence_directory: Option<PathBuf>,

    #[arg(env)]
    tracing_directory: Option<PathBuf>,
    #[arg(env, default_value = "")]
    tracing_file_name_prefix: String,
    #[arg(env, default_value = "log")]
    tracing_file_name_suffix: String,
    #[arg(env, default_value_t = 64)]
    tracing_max_log_files: usize,
}

async fn real_main(
    Args {
        persistence_directory,
        tracing_directory,
        tracing_file_name_prefix,
        tracing_file_name_suffix,
        tracing_max_log_files,
    }: Args,
    home_assistant: HomeAssistant,
) -> ! {
    let tracing_to_directory_res = tracing_directory
        .map(|tracing_directory| {
            tracing_appender::rolling::Builder::new()
                .filename_prefix(tracing_file_name_prefix)
                .filename_suffix(tracing_file_name_suffix)
                .max_log_files(tracing_max_log_files)
                .build(tracing_directory)
                .map(tracing_appender::non_blocking)
        })
        .transpose();

    let (tracing_to_directory, _guard, tracing_to_directory_initialization_error) =
        match tracing_to_directory_res {
            Ok(tracing_to_directory) => match tracing_to_directory {
                Some((tracing_to_directory, guard)) => {
                    (Some(tracing_to_directory), Some(guard), None)
                }
                None => (None, None, None),
            },
            Err(error) => (None, None, Some(error)),
        };

    registry()
        .with(
            fmt::layer()
                .pretty()
                .with_span_events(FmtSpan::ACTIVE)
                .with_filter(LevelFilter::from_level(Level::TRACE)),
        )
        .with(TracingToHomeAssistant)
        .with(tracing_to_directory.map(|writer| {
            fmt::layer()
                .pretty()
                .with_span_events(FmtSpan::ACTIVE)
                .with_writer(writer)
                .with_filter(LevelFilter::from_level(Level::TRACE))
        }))
        .init();

    if let Some(error) = tracing_to_directory_initialization_error {
        tracing::error!(?error, "cannot trace to directory");
    }

    let built_at = build_info::BUILD_TIME;
    tracing::info!(built_at);

    // let lamp = HomeAssistantLight {
    //     home_assistant,
    //     object_id: ObjectId::from_str("jacob_s_lamp_side").unwrap(),
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

        let is_on = some_light.is_on().await;
        tracing::info!(?is_on);
        let is_off = some_light.is_off().await;
        tracing::info!(?is_off);

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
    let args = Args::parse();

    pyo3_async_runtimes::tokio::future_into_py::<_, ()>(py, async {
        real_main(args, home_assistant).await;
    })
}

/// A Python module implemented in Rust.
#[pymodule]
fn smart_home_in_rust_with_home_assistant(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_function(wrap_pyfunction!(main, module)?)?;
    Ok(())
}
