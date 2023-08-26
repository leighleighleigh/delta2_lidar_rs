#![allow(unused_imports)]

use std::alloc::System;
use std::io::{self, Write};
use std::time::Duration;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::Result;
use clap::{arg, command, value_parser};
use log::{error, info};
use std::sync::mpsc::channel;
// use std::thread;

mod protocol;
use crate::protocol::{MeasurementFrame, PartialFrame};

mod lidar;
use crate::lidar::Lidar;

use rerun::components::{ColorRGBA, Point3D, Radius, Transform3D, Vec3D};
use rerun::transform::*;

fn main() -> Result<()> {
    env_logger::init();

    let matches = command!() 
        .arg(arg!([PORT]).value_parser(value_parser!(PathBuf)).default_value("/dev/ttyUSB0"))
        .get_matches();

    let recording = rerun::RecordingStreamBuilder::new("Delta2")
        .connect(rerun::default_server_addr(), None)
        .unwrap();


    let serial_port : PathBuf = matches.get_one::<PathBuf>("PORT").unwrap().into();
    let port_str : String = serial_port.to_string_lossy().to_string();
    let mut delta = Lidar::new();
    
    delta.open(port_str).unwrap();

    loop {
        match delta.recv() {
            Ok(frame) => {
                // a 180 degree offset means the motor is at the 'back' of the module, which makes more sense to me.
                let start_angle = frame.start_angle + 180.0; 
                let offset = frame.offset_angle;

                let start_angles = frame
                    .measurements
                    .iter()
                    .enumerate()
                    .map(|(i, _dist)| {
                        let angle = start_angle + (i as f32) * offset;
                        // Point3D::new(dx,dy, 0.0)
                        Transform3D::new(TranslationRotationScale3D::rigid(
                            Vec3D::ZERO,
                            RotationAxisAngle::new(
                                Vec3D::new(0.0, 0.0, 1.0),
                                Angle::Degrees(-angle + 90.0),
                            ),
                        ))
                    })
                    .collect::<Vec<_>>();


                let range_data = frame
                    .measurements
                    .iter()
                    .enumerate()
                    .map(|(i, dist)| {
                        let angle = start_angle + (i as f32) * offset;
                        let dx = dist.distance_mm * angle.to_radians().sin() / 1000.0;
                        let dy = dist.distance_mm * angle.to_radians().cos() / 1000.0;
                        Point3D::new(dx, dy, 0.0)
                    })
                    .collect::<Vec<_>>();

                let quality_data = frame
                    .measurements
                    .iter()
                    .enumerate()
                    .map(|(_i, dist)| {
                        ColorRGBA::from_rgb(dist.signal_quality, 0, 0)
                    })
                    .collect::<Vec<_>>();
                    
                let t = SystemTime::now().duration_since(UNIX_EPOCH).expect("Time went backwards").as_nanos();

                recording.set_time_nanos("scan", t as i64);
                recording.set_time_nanos("tf", t as i64);

                rerun::MsgSender::new("tf")
                    .with_component(&start_angles)?
                    .send(&recording)?;

                rerun::MsgSender::new("scan")
                    .with_component(&range_data)?
                    .with_component(&quality_data)?
                    .with_splat(Radius(0.025))?
                    .send(&recording)?;
            }
            Err(e) => {
                panic!("{}", e);
            }
        }
    }
}
