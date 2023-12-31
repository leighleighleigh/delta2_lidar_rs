use protocol::{MeasurementFrame,Measurement, FullScan};

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

#[pyclass]
#[pyo3{name = "FullScan"}]
#[derive(Clone)]
struct PyFullScan {
    scan: FullScan,
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

    fn read_frame(&mut self) -> PyResult<PyMeasurementFrame> {
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

    fn read_full_scan(&mut self) -> PyResult<PyFullScan> {
        // reads a frame, or returns a None object
        match self.dev.recv_fullscan() {
            Ok(msg) => {
                // need to turn into pyobject
                let pymsg : PyFullScan = PyFullScan { scan: msg };
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
    fn sector_angle(&self) -> PyResult<f32> {
        Ok(self.frame.sector_angle())
    }

    #[getter]
    fn end_angle(&self) -> PyResult<f32> {
        let s = self.frame.start_angle;
        let o = (self.frame.measurements.len() as f32) * self.frame.offset_angle;
        Ok(s + o)
    }

    #[getter]
    fn timestamp(&self) -> PyResult<u128> {
        Ok(self.frame.timestamp)
    }

    #[getter]
    fn measurements(&self) -> PyResult<Vec<PyMeasurement>> {
        Ok(self.frame.measurements.iter().map(|m|{ PyMeasurement{m:m.clone()}}).collect::<Vec<PyMeasurement>>())
    }

    #[getter]
    fn points(&self) -> PyResult<Vec<(f32,f32)>> {
        Ok(self.frame.points())
    }

    fn __str__(&self) -> PyResult<String> {
        Ok(self.frame.to_string())
    }

    fn as_json(&self) -> PyResult<String> {
        Ok(self.frame.as_json())
    }
}

#[pymethods]
impl PyFullScan {
    #[getter]
    fn frames(&self) -> PyResult<Vec<PyMeasurementFrame>> {
        Ok(self.scan.frames.iter().map(|f| PyMeasurementFrame{frame:f.clone()}).collect::<Vec<PyMeasurementFrame>>())
    }

    #[getter]
    fn points(&self) -> PyResult<Vec<(f32,f32)>> {
        Ok(self.scan.points())
    }

    #[getter]
    fn timestamp_range(&self) -> PyResult<i64> {
        Ok(self.scan.timestamp_range())
    }

    #[getter]
    fn timestamp(&self) -> PyResult<u128> {
        Ok(self.scan.timestamp())
    }

    #[getter]
    fn rpm(&self) -> PyResult<f32> {
        Ok(self.scan.rpm())
    }

    #[getter]
    fn complete(&self) -> PyResult<bool> {
        Ok(self.scan.complete())
    }

    fn __str__(&self) -> PyResult<String> {
        Ok(self.scan.to_string())
    }
    
    fn as_json(&self) -> PyResult<String> {
        Ok(self.scan.as_json())
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

    #[getter]
    fn point(&self) -> PyResult<(f32,f32)> {
        Ok(self.m.point())
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
    m.add_class::<PyFullScan>()?;
    Ok(())
}


