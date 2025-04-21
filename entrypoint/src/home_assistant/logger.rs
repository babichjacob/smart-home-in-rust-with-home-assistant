use crate::python_utils::{detach, validate_type_by_name};
use arbitrary_value::{arbitrary::Arbitrary, map::Map};
use once_cell::sync::OnceCell;
use pyo3::{prelude::*, types::PyTuple};

#[derive(Debug)]
pub struct HassLogger(Py<PyAny>);

impl<'source> FromPyObject<'source> for HassLogger {
    fn extract_bound(ob: &Bound<'source, PyAny>) -> PyResult<Self> {
        // region: Validation
        validate_type_by_name(ob, "HassLogger")?;
        // endregion: Validation

        Ok(Self(detach(ob)))
    }
}

#[derive(Debug, Clone, IntoPyObject)]
pub struct LogData<ExcInfo> {
    /// If exc_info does not evaluate as false, it causes exception information to be added to the logging message.
    /// If an exception tuple (in the format returned by sys.exc_info()) or an exception instance is provided, it is used;
    /// otherwise, sys.exc_info() is called to get the exception information.
    exc_info: Option<ExcInfo>,

    /// If true, stack information is added to the logging message, including the actual logging call.
    /// Note that this is not the same stack information as that displayed through specifying exc_info:
    /// The former is stack frames from the bottom of the stack up to the logging call in the current thread,
    /// whereas the latter is information about stack frames which have been unwound,
    /// following an exception, while searching for exception handlers.
    ///
    /// You can specify stack_info independently of exc_info,
    /// e.g. to just show how you got to a certain point in your code, even when no exceptions were raised.
    /// The stack frames are printed following a header line which says:
    ///
    /// Stack (most recent call last):
    ///
    /// This mimics the `Traceback (most recent call last):` which is used when displaying exception frames.
    stack_info: bool,

    /// If greater than 1, the corresponding number of stack frames are skipped
    /// when computing the line number and function name set in the LogRecord created for the logging event.
    /// This can be used in logging helpers so that the function name, filename and line number recorded
    /// are not the information for the helper function/method, but rather its caller.
    stacklevel: u16,

    /// This can be used to pass a dictionary which is used to populate the __dict__ of the LogRecord
    /// created for the logging event with user-defined attributes.
    /// These custom attributes can then be used as you like.
    /// For example, they could be incorporated into logged messages.
    extra: Map,
}

impl HassLogger {
    pub fn new(py: Python<'_>, name: &str) -> PyResult<Self> {
        static LOGGING_MODULE: OnceCell<Py<PyModule>> = OnceCell::new();

        let logging_module = LOGGING_MODULE
            .get_or_try_init(|| Result::<_, PyErr>::Ok(py.import("logging")?.unbind()))?
            .bind(py);
        let logger = logging_module.call_method1("getLogger", (name,))?;

        Ok(logger.extract()?)
    }

    pub fn debug<'py, ExcInfo: IntoPyObject<'py>>(
        &self,
        py: Python<'py>,
        msg: &str,
        args: Vec<Arbitrary>,
        log_data: Option<LogData<ExcInfo>>,
    ) -> PyResult<()> {
        let mut all_args = vec![msg.into_pyobject(py)?.into_any()];
        for arg in args {
            let arg = arg.into_pyobject(py)?;
            all_args.push(arg);
        }
        let all_args = PyTuple::new(py, all_args)?;

        let kwargs = log_data
            .map(|log_data| log_data.into_pyobject(py))
            .transpose()?;

        self.0.call_method(py, "debug", all_args, kwargs.as_ref())?;

        Ok(())
    }

    pub fn info<'py, ExcInfo: IntoPyObject<'py>>(
        &self,
        py: Python<'py>,
        msg: &str,
        args: Vec<Arbitrary>,
        log_data: Option<LogData<ExcInfo>>,
    ) -> PyResult<()> {
        let mut all_args = vec![msg.into_pyobject(py)?.into_any()];
        for arg in args {
            let arg = arg.into_pyobject(py)?;
            all_args.push(arg);
        }
        let all_args = PyTuple::new(py, all_args)?;

        let kwargs = log_data
            .map(|log_data| log_data.into_pyobject(py))
            .transpose()?;

        self.0.call_method(py, "info", all_args, kwargs.as_ref())?;

        Ok(())
    }

    pub fn warning<'py, ExcInfo: IntoPyObject<'py>>(
        &self,
        py: Python<'py>,
        msg: &str,
        args: Vec<Arbitrary>,
        log_data: Option<LogData<ExcInfo>>,
    ) -> PyResult<()> {
        let mut all_args = vec![msg.into_pyobject(py)?.into_any()];
        for arg in args {
            let arg = arg.into_pyobject(py)?;
            all_args.push(arg);
        }
        let all_args = PyTuple::new(py, all_args)?;

        let kwargs = log_data
            .map(|log_data| log_data.into_pyobject(py))
            .transpose()?;

        self.0
            .call_method(py, "warning", all_args, kwargs.as_ref())?;

        Ok(())
    }

    pub fn error<'py, ExcInfo: IntoPyObject<'py>>(
        &self,
        py: Python<'py>,
        msg: &str,
        args: Vec<Arbitrary>,
        log_data: Option<LogData<ExcInfo>>,
    ) -> PyResult<()> {
        let mut all_args = vec![msg.into_pyobject(py)?.into_any()];
        for arg in args {
            let arg = arg.into_pyobject(py)?;
            all_args.push(arg);
        }
        let all_args = PyTuple::new(py, all_args)?;

        let kwargs = log_data
            .map(|log_data| log_data.into_pyobject(py))
            .transpose()?;

        self.0.call_method(py, "error", all_args, kwargs.as_ref())?;

        Ok(())
    }

    pub fn critical<'py, ExcInfo: IntoPyObject<'py>>(
        &self,
        py: Python<'py>,
        msg: &str,
        args: Vec<Arbitrary>,
        log_data: Option<LogData<ExcInfo>>,
    ) -> PyResult<()> {
        let mut all_args = vec![msg.into_pyobject(py)?.into_any()];
        for arg in args {
            let arg = arg.into_pyobject(py)?;
            all_args.push(arg);
        }
        let all_args = PyTuple::new(py, all_args)?;

        let kwargs = log_data
            .map(|log_data| log_data.into_pyobject(py))
            .transpose()?;

        self.0
            .call_method(py, "critical", all_args, kwargs.as_ref())?;

        Ok(())
    }
}
