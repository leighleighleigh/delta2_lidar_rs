#![allow(unused_imports)]
use anyhow::Result;
use log::{error, info};

pub mod protocol;
// use crate::protocol::{MeasurementFrame, PartialFrame};

pub mod lidar;
use crate::lidar::Lidar;

extern crate pyo3;

use pyo3::exceptions::{PyOSError, PyRuntimeError, PyValueError, PyTypeError};
use pyo3::prelude::*;
use pyo3::types::{PyModule, PyList};
use pyo3::{PyResult};


#[pyclass]
#[pyo3{name = "Lidar"}]
struct PyLidar {
    dev: Lidar,
}

#[pymethods]
impl PyLidar {
    #[new]
    fn new() -> PyResult<Self> {
        let bus = Lidar::new();
        Ok(PyLidar{dev: bus})
    }

    fn open(&mut self, port : String) -> PyResult<()> {
        self.dev.open(port).map_err(|e| {
            PyOSError::new_err(format!("{}", e))
        })?;
        Ok(())
    }
}


#[pymodule]
fn delta2_lidar_py(_py: Python<'_>, m: &PyModule) -> PyResult<()> {
    m.add_class::<PyLidar>()?;

    Ok(())
}


