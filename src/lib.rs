use protocol::{MeasurementFrame,Measurement};

pub mod protocol;
pub mod lidar;
use crate::lidar::Lidar;

extern crate pyo3;

use pyo3::exceptions::{PyOSError};
use pyo3::prelude::*;
use pyo3::types::{PyModule};
use pyo3::PyResult;


#[pyclass]
#[pyo3{name = "Lidar"}]
struct PyLidar {
    dev: Lidar,
}

#[pyclass]
#[pyo3{name = "MeasurementFrame"}]
#[derive(Clone)]
struct PyMeasurementFrame {
    frame: MeasurementFrame,
}

#[pyclass]
#[pyo3{name = "Measurement"}]
#[derive(Clone)]
struct PyMeasurement{
    m: Measurement,
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

    fn read(&mut self) -> PyResult<PyMeasurementFrame> {
        // reads a frame, or returns a None object
        match self.dev.recv() {
            Ok(msg) => {
                // need to turn into pyobject
                let pymsg : PyMeasurementFrame = PyMeasurementFrame { frame: msg };
                Ok(pymsg)
            },
            Err(e) => {
                Err(PyOSError::new_err(format!("{}",e)))
            }
        }
    }

    fn alive(&self) -> bool {
        self.dev.alive()
    }
}

#[pymethods]
impl PyMeasurementFrame {
    #[new]
    fn new() -> PyResult<Self> {
        let m = MeasurementFrame::default();
        Ok(PyMeasurementFrame{frame: m})
    }

    #[getter]
    fn rpm(&self) -> PyResult<f32> {
        Ok(self.frame.rpm)
    }

    #[getter]
    fn offset_angle(&self) -> PyResult<f32> {
        Ok(self.frame.offset_angle)
    }

    #[getter]
    fn start_angle(&self) -> PyResult<f32> {
        Ok(self.frame.start_angle)
    }

    #[getter]
    fn timestamp(&self) -> PyResult<u128> {
        Ok(self.frame.timestamp)
    }

    #[getter]
    fn measurements(&self) -> PyResult<Vec<PyMeasurement>> {
        Ok(self.frame.measurements.iter().map(|m|{ PyMeasurement{m:m.clone()}}).collect::<Vec<PyMeasurement>>())
    }

    fn __str__(&self) -> PyResult<String> {
        Ok(self.frame.to_string())
    }

    fn as_json(&self) -> PyResult<String> {
        Ok(self.frame.as_json())
    }
}

#[pymethods]
impl PyMeasurement {
    #[getter]
    fn angle(&self) -> PyResult<f32> {
        Ok(self.m.angle)
    }

    #[getter]
    fn signal_quality(&self) -> PyResult<u8> {
        Ok(self.m.signal_quality)
    }

    #[getter]
    fn distance_mm(&self) -> PyResult<f32> {
        Ok(self.m.distance_mm)
    }

    fn __str__(&self) -> PyResult<String> {
        Ok(self.m.to_string())
    }
}

#[pymodule]
fn delta2_lidar_py(_py: Python<'_>, m: &PyModule) -> PyResult<()> {
    m.add_class::<PyLidar>()?;
    m.add_class::<PyMeasurementFrame>()?;
    m.add_class::<PyMeasurement>()?;
    Ok(())
}


