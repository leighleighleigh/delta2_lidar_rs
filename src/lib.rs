#![allow(unused_imports)]
use std::alloc::System;
use std::io::{self, Write};
use std::time::Duration;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::Result;
use log::{error, info};
use std::sync::mpsc::channel;
// use std::thread;

pub mod protocol;
use crate::protocol::{MeasurementFrame, PartialFrame};

pub mod lidar;
use crate::lidar::Lidar;

