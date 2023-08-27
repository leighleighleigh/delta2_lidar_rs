#![allow(unused_imports)]

use anyhow::Result;
use log::{error, info, warn};
use std::alloc::System;
use std::io::{self, Write};
use std::time::Duration;
use std::time::{SystemTime, UNIX_EPOCH};

// the actual library
use delta2_lidar_rs::lidar::Lidar;
use delta2_lidar_rs::protocol::{MeasurementFrame, Measurement};

// rerun application logging
use rerun::components::{ColorRGBA, Point3D, Radius, Transform3D, Vec3D};
use rerun::transform::*;

fn main() -> Result<()> {
    env_logger::init();

    // start a rerun recording session, streaming over websockets
    let recording = rerun::RecordingStreamBuilder::new("Delta2")
        .connect(rerun::default_server_addr(), None)
        .unwrap();


    // create a lidar device
    let mut delta = Lidar::new();
    delta.open("/dev/ttyUSB0".to_string()).unwrap();

    // read data!
    loop {
        match delta.recv() {
            Ok(frame) => {
                // with improvements to the Measurement struct, we can now just zip through and convert each measurement to a 
                // Transform3D, Point3D, etc etc.
                // much cleaner!
                let stamp = frame.timestamp as i64;

                // set the time for this data
                recording.set_time_nanos("scan", stamp);
                recording.set_time_nanos("tf", stamp);

                // log the RPM of the lidar
                let rpm = rerun::components::Scalar(frame.rpm as f64);

                // transform data
                let sweep = frame.measurements.iter().map(|m| {
                        // idk why but the angle needs a +90 and -ve, in rerun.
                        Transform3D::new(TranslationRotationScale3D::rigid(
                            Vec3D::ZERO,
                            RotationAxisAngle::new(
                                Vec3D::new(0.0, 0.0, 1.0),
                                Angle::Degrees(-m.angle + 90.0),
                            ),
                        ))
                    }).collect::<Vec<_>>();

                // scan data
                let (ranges, quality): (Vec<Point3D>,Vec<ColorRGBA>) = frame.measurements.iter().map(|m| {
                        let dx = m.distance_mm * m.angle.to_radians().sin() / 1000.0;
                        let dy = m.distance_mm * m.angle.to_radians().cos() / 1000.0;

                        (Point3D::new(dx, dy, 0.0),ColorRGBA::from_rgb(m.signal_quality, 0, 0))
                }).unzip();

                // log the speed of the spinny spin
                rerun::MsgSender::new("scan/rpm")
                    .with_component(&vec![rpm])?
                    .send(&recording)?;

                // log the direction of the spinny spin
                rerun::MsgSender::new("tf")
                    .with_component(&sweep)?
                    .send(&recording)?;

                // log the range measurements and signal quality of the spinny spin
                rerun::MsgSender::new("scan")
                    .with_component(&ranges)?
                    .with_component(&quality)?
                    .with_splat(Radius(0.01))?
                    .send(&recording)?;
            }
            Err(e) => {
                warn!("{}", e);
            }
        }
    }
}
