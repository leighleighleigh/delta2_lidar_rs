use crate::protocol::{MeasurementFrame, PartialFrame};

use anyhow::Result;
use log::{debug, error, warn};

use serialport::{self, SerialPort};

use std::io::{self, Write};

use std::sync::mpsc::channel;
use std::sync::mpsc::{self, RecvError};

use std::thread;
use std::time::Duration;

#[derive(Default)]
pub struct Lidar {
    pub rx: Option<mpsc::Receiver<PartialFrame>>,
    // A handle to the background receiver thread is put here
    worker_handle: Option<thread::JoinHandle<Result<()>>>,
}

impl Lidar {
    // new creates an inactive Lidar object.
    // useful if you want to set it up as a typed variable for later on!
    pub fn new() -> Lidar {
        Lidar::default()
    }

    pub fn alive(&self) -> bool {
        // checks if worker thread is alive, and the channels exist.
        match self.rx.is_some() && self.worker_handle.is_some() {
            true => {
                let th : &thread::JoinHandle<Result<()>> = self.worker_handle.as_ref().unwrap();
                !th.is_finished()
            }
            false => false,
        }
    }

    pub fn recv(&mut self) -> Result<MeasurementFrame, RecvError> {
        match self.alive() {
            true => match self.rx.as_ref().unwrap().recv() {
                Ok(m) => {
                    if m.is_measurement_type() {
                        Ok(m.into())
                    } else {
                        Err(RecvError)
                    }
                }
                Err(_) => Err(RecvError),
            },
            false => Err(RecvError),
        }
    }

    // attempts to bind to the serial port provided by <path>,
    // if successful, sets up the message passing channel,
    // and begins reading data in a background thread.
    pub fn open(&mut self, path: String) -> Result<(), Box<dyn std::error::Error>> {
        if self.alive() {
            warn!("Lidar has already been opened! This may cause unexpected behaviour!");
        }

        // Channel is used to pass decoded frames from worker thread -> main thread
        let (tx, rx) = channel();
        self.rx = Some(rx);

        self.worker_handle = Some(
            thread::Builder::new()
                .name("lidar_decode_thread".to_string())
                .spawn(move || {
                    let port_name = path.as_str();
                    let port_builder =
                        serialport::new(port_name, 115200).timeout(Duration::from_millis(20));

                    let port: Option<Box<dyn SerialPort>> = match port_builder.open() {
                        Ok(p) => {
                            // We can continue
                            Some(p)
                        }
                        Err(err) => {
                            // retry!
                            panic!("{}", err);
                        }
                    };

                    if let Some(mut serial) = port {
                        let mut serial_data_buffer: Vec<u8> = vec![];
                        let mut serial_temp_buf: Vec<u8> = vec![0; 256];

                        // set the new frame
                        let mut new_frame = PartialFrame::new();

                        // continuously read new frames
                        loop {
                            match serial.read(serial_temp_buf.as_mut_slice()) {
                                Ok(t) => {
                                    // concat the serial read buffer onto the serial accumualted buffer
                                    serial_data_buffer.append(&mut serial_temp_buf[..t].to_vec());

                                    let j = serial_data_buffer.len(); // starting size of data buffer
                                    let result = new_frame.write_all(&serial_data_buffer[..j]);
                                    let written = new_frame.bytes_written;

                                    // if we have a buffer full error, then we need to remove not 'j', but 'j-n' bytes from the data buffer.
                                    let n = j - written;
                                    debug!("avail = {}, written = {}, remain = {}", j, written, n);
                                    serial_data_buffer =
                                        serial_data_buffer.as_mut_slice()[..n].to_vec();

                                    match result.as_ref() {
                                        Ok(_) => {}
                                        Err(err) => {
                                            match err.raw_os_error() {
                                                Some(105) => {
                                                    // no space left in frame 'buffer'
                                                    debug!(
                                                        "Frame full. t = {}, frame = {}",
                                                        t,
                                                        new_frame.data.len()
                                                    );
                                                }
                                                Some(_) => {}
                                                None => {}
                                            }
                                        }
                                    }

                                    // if the frame is done
                                    if new_frame.finished() {
                                        let send_result = tx.send(new_frame.clone());

                                        match send_result {
                                            Ok(_) => {}
                                            Err(e) => {
                                                error!("{}", e);
                                            }
                                        };
                                    }

                                    // as mentioned, an error on the frame write result means it's buffer is full.
                                    // so we reset.
                                    if result.is_err() {
                                        new_frame.reset();
                                    }
                                }
                                // ignore timeout
                                Err(ref e) if e.kind() == io::ErrorKind::TimedOut => (),
                                // all other errors cause panic
                                Err(e) => {
                                    panic!("{}", e)
                                }
                            }
                        }
                    }

                    // all good! :D
                    Ok(())
                })?,
        );
        Ok(())
    }
}
