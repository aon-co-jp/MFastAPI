//! Python extension `rmojo_py`: run a function of Mojo-style source in RMojo (no Mojo toolchain, no C++).
//! The program lives on one dedicated big-stack worker thread (RMojo values are not `Send`); a call sends the scalar
//! arguments over a channel and waits with the GIL released, so other Python threads keep running.
use pyo3::exceptions::{PyRuntimeError, PyTimeoutError, PyValueError};
use pyo3::prelude::*;
use pyo3::types::{PyBool, PyFloat, PyInt, PyString};
use rmojo::{Limits, Value};
use std::sync::mpsc::{channel, Sender};
use std::sync::Mutex;

type Job = (String, Vec<Value>, Limits, Sender<Result<Value, String>>);

#[pyclass(frozen, name = "Handler")]
struct Handler {
    tx: Mutex<Sender<Job>>,
    max_steps: u64,
    max_ms: Option<u64>,
}

fn to_value(o: &Bound<'_, PyAny>) -> PyResult<Value> {
    if o.is_none() {
        Ok(Value::Null)
    } else if o.is_instance_of::<PyBool>() {
        Ok(Value::Bool(o.extract()?))
    } else if o.is_instance_of::<PyInt>() {
        Ok(Value::Int(o.extract()?))
    } else if o.is_instance_of::<PyFloat>() {
        Ok(Value::Float(o.extract()?))
    } else if o.is_instance_of::<PyString>() {
        Ok(Value::Str(o.extract()?))
    } else {
        Err(PyValueError::new_err("only None, bool, int, float and str can cross into RMojo"))
    }
}

#[pymethods]
impl Handler {
    /// Compile `source` once. Raises ValueError on a compile error.
    #[new]
    #[pyo3(signature = (source, max_steps = 50_000_000, max_ms = None))]
    fn new(source: &str, max_steps: u64, max_ms: Option<u64>) -> PyResult<Self> {
        let (tx, rx) = channel::<Job>();
        let (ready_tx, ready_rx) = channel::<Result<(), String>>();
        let src = source.to_string();
        std::thread::Builder::new()
            .stack_size(1 << 28)
            .spawn(move || {
                let prog = match rmojo::compile_source(&src) {
                    Ok(p) => {
                        let _ = ready_tx.send(Ok(()));
                        p
                    }
                    Err(e) => {
                        let _ = ready_tx.send(Err(e));
                        return;
                    }
                };
                while let Ok((name, args, lim, reply)) = rx.recv() {
                    let _ = reply.send(prog.call_limited(&name, &args, &lim));
                }
            })
            .map_err(|e| PyRuntimeError::new_err(e.to_string()))?;
        match ready_rx.recv() {
            Ok(Ok(())) => Ok(Handler { tx: Mutex::new(tx), max_steps, max_ms }),
            Ok(Err(e)) => Err(PyValueError::new_err(e)),
            Err(_) => Err(PyRuntimeError::new_err("rmojo worker failed to start")),
        }
    }

    /// Call function `name`. LimitError -> TimeoutError, other handler errors -> ValueError.
    #[pyo3(signature = (name, *args))]
    fn call(&self, py: Python<'_>, name: &str, args: &Bound<'_, pyo3::types::PyTuple>) -> PyResult<Py<PyAny>> {
        let vals = args.iter().map(|a| to_value(&a)).collect::<PyResult<Vec<_>>>()?;
        let lim = Limits { max_steps: self.max_steps, max_ms: self.max_ms };
        let (rtx, rrx) = channel();
        let job: Job = (name.to_string(), vals, lim, rtx);
        let sent = self.tx.lock().map(|t| t.send(job).is_ok()).unwrap_or(false);
        if !sent {
            return Err(PyRuntimeError::new_err("rmojo worker unavailable"));
        }
        let res = py.detach(move || rrx.recv());
        match res {
            Ok(Ok(v)) => Ok(match v {
                Value::Int(i) => i.into_pyobject(py)?.into_any().unbind(),
                Value::Float(f) => f.into_pyobject(py)?.into_any().unbind(),
                Value::Bool(b) => PyBool::new(py, b).to_owned().into_any().unbind(),
                Value::Str(s) => s.into_pyobject(py)?.into_any().unbind(),
                Value::Null => py.None(),
            }),
            Ok(Err(e)) if e.starts_with("LimitError") => Err(PyTimeoutError::new_err(e)),
            Ok(Err(e)) => Err(PyValueError::new_err(e)),
            Err(_) => Err(PyRuntimeError::new_err("rmojo worker unavailable")),
        }
    }
}

#[pymodule]
fn rmojo_py(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<Handler>()?;
    Ok(())
}
