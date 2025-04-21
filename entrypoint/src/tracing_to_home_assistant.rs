use std::fmt::Debug;

use pyo3::Python;
use tracing::{
    field::{Field, Visit},
    Event, Level, Subscriber,
};
use tracing_subscriber::{layer::Context, Layer};

use crate::home_assistant::logger::{HassLogger, LogData};

pub struct TracingToHomeAssistant;

impl<S: Subscriber> Layer<S> for TracingToHomeAssistant {
    fn on_event(&self, event: &Event<'_>, _ctx: Context<'_, S>) {
        let meta = event.metadata();
        let file = meta.file();
        let level = meta.level();
        let line = meta.line();
        let target = meta.target();

        let mut msg = String::new();

        struct StringVisitor<'a> {
            s: &'a mut String,
        }
        impl<'a> Visit for StringVisitor<'a> {
            fn record_debug(&mut self, field: &Field, value: &dyn Debug) {
                let field_name = field.name();
                if field_name != "message" {
                    self.s.push_str(field_name);
                    self.s.push_str(" = ");
                }
                self.s.push_str(&format!("{value:?}"));
                self.s.push_str(", ");
            }
        }
        let mut visitor = StringVisitor { s: &mut msg };
        event.record(&mut visitor);

        if let Some(file) = file {
            msg.push_str(&format!("at {file}"));
            if let Some(line) = line {
                msg.push_str(&format!(":{line}"));
            }
        }

        let args = vec![];

        let log_data: Option<LogData<()>> = None;

        Python::with_gil(|py| {
            let Ok(hass_logger) = HassLogger::new(py, target) else {
                return;
            };

            if *level == Level::TRACE {
                // Errors are ignored because there's nowhere to report them besides
                // through the tracer itself!
                let _ = hass_logger.debug(py, &msg, args, log_data);
            } else if *level == Level::DEBUG {
                let _ = hass_logger.debug(py, &msg, args, log_data);
            } else if *level == Level::INFO {
                let _ = hass_logger.info(py, &msg, args, log_data);
            } else if *level == Level::WARN {
                let _ = hass_logger.warning(py, &msg, args, log_data);
            } else if *level == Level::ERROR {
                let _ = hass_logger.error(py, &msg, args, log_data);
            } else {
                unreachable!("those are all 5 levels");
            }
        });
    }
}
