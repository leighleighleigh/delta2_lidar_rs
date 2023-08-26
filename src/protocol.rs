// Aliexpress 'DELTA-2 Roomba Lidar' protocol parser.
// Product listing: https://www.aliexpress.com/item/1005004140103483.html
//
// Reverse-engineering credit goes to 'notblackmagic': https://notblackmagic.com/bitsnpieces/lidar-modules/
//
// Code here was originally prototyped in Python, to confirm the protocol structure, then ported to Rust as a state-machine implementation.
// Leigh Oliver, August 13th 2023.

// Specific design goals for rust port
// - improve 'glitched' data decoding, by having multiple 'partial frames' all being assembled simultaneously.
// - ideally, be no_std compatible, so that this state machine / protocol structs can be used on microcontrollers too.
// - be serializable to CSV / JSON for data capture

// DATA FRAME STRUCTURE (DELTA-2)
// Byte#: 0       1 & 2   3        4    5        6 & 7    8          -> N-3 	    N-2 	N-1
// Desc : Header  Length  Protocol Type Command  Payload  Length [N] 	Payload 	CRC 	CRC
use itertools::Itertools;
// This type+enum is used to keep track of what bytes mean what, and how many bytes each
// part of the frame is. I've made a simple tuple type to represent this, hopefully it will make
// the state machine code much cleaner! (no idea tho).
// offset, length, expected constnat value
use log::debug;
use std::{io::{Error, Write}, fmt::Display};
use serde::Serialize;
use std::time::{SystemTime, UNIX_EPOCH};

// These are the hard-coded 'magic numbers' which we expect to receive in each frame.
// const FRAME_HEADER : FramePart = FramePart{ offset: 0, length: 1, expected: Some(0xAA) };
// const FRAME_LENGTH : FramePart = FramePart{ offset: 1, length: 2, expected: None };
// const FRAME_VERSION : FramePart = FramePart{ offset: 3, length: 1, expected: Some(0x01) };
// const FRAME_TYPE : FramePart = FramePart{ offset: 4, length: 1, expected: Some(0x61) };
// const FRAME_CMD : FramePart = FramePart{ offset: 5, length: 1, expected: None };
//   const FRAME_CMD_DEVICEHEALTH : FramePart = FramePart{ offset: 5, length: 1, expected: Some(0xAE) };
//   const FRAME_CMD_MEASUREMENT : FramePart = FramePart{ offset: 5, length: 1, expected: Some(0xAD) };
// const FRAME_PAYLOAD_LENGTH : FramePart = FramePart{ offset: 6, length: 2, expected: None };
// starts at byte 8, ends at LEN-3.
// const FRAME_PAYLOAD : FramePart = FramePart{ offset: 8, length: -3, expected: None };
// starts at LEN-2, ends at LEN.
// const FRAME_CRC : FramePart = FramePart{ offset: -2, length: 2, expected: None };

#[derive(Debug, Clone)]
pub struct PartialFrame {
    pub data: Vec<u8>, // buffered input data gets copied here, to be decoded at the end
    pub bytes_wanted: usize, // how many bytes to read until we complete the next part
    pub bytes_written: usize, // delta 
    pub timestamp: u128, // unix epoch nanoseconds when the header was identified.
}

#[derive(Serialize)]
#[derive(Debug, Clone)]
pub struct Measurement {
    pub angle : f32, // degrees
    pub signal_quality: u8,
    pub distance_mm : f32,
}

impl Display for Measurement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!("{:3.1} deg - {:3.1} cm",self.angle,self.distance_mm/10.0))
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct MeasurementFrame {
    pub rpm: f32,
    pub offset_angle: f32,
    pub start_angle: f32,
    pub timestamp: u128, // unix epoch nanoseconds when the header was identified.
    pub measurements: Vec<Measurement>,
}

impl Default for Measurement {
    fn default() -> Self {
        Measurement {
            angle: 0.0,
            signal_quality: 0,
            distance_mm: 0.0,
        }
    }
}

impl Measurement {
    pub fn point(&self) -> (f32,f32) {
        // returns the data in cartesian metre units
        let dx = self.distance_mm * self.angle.to_radians().sin() / 1000.0;
        let dy = self.distance_mm * self.angle.to_radians().cos() / 1000.0;
        (dx,dy)
    }
}

impl MeasurementFrame {
    pub fn as_json(&self) -> String {
        serde_json::to_string(&self).expect("Serialized to JSON")
    }

    pub fn points(&self) -> Vec<(f32,f32)> {
        // calls .cartesian on all measurements, returning a 'point cloud'
        self.measurements.iter().map(|m| m.point()).collect_vec()
    }
}

impl Display for MeasurementFrame {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!("{} rpm, {:3.1} deg, {} pts",self.rpm,self.start_angle,self.measurements.len()))
    }
}

impl Default for MeasurementFrame {
    fn default() -> Self {
        MeasurementFrame {
            rpm: 0.0,
            offset_angle: 0.0,
            start_angle: 0.0,
            timestamp: 0,
            measurements: vec![Measurement::default()],
        }
    }
}

impl From<PartialFrame> for MeasurementFrame {
    fn from(value: PartialFrame) -> Self {
        if !value.is_measurement_type() || !value.finished() {
            MeasurementFrame::default()
        } else {
            let data_start: usize = 8;
            let rpm_raw: u8 = value.data.as_slice()[data_start];
            let rpm : f32 = (rpm_raw as f32) * 3.0;

            // assemble offset angle
            let off_msb: u8 = value.data.as_slice()[data_start + 1];
            let off_lsb: u8 = value.data.as_slice()[data_start + 2];
            let _offset_angle = u16::from_be_bytes([off_msb, off_lsb]);

            // assemble start angle
            let start_msb: u8 = value.data.as_slice()[data_start + 3];
            let start_lsb: u8 = value.data.as_slice()[data_start + 4];
            let start_angle = u16::from_be_bytes([start_msb, start_lsb]);
            
            // OLD: Offset angle was read from data.
            // let offset_angle_deg : f32 = (offset_angle as f32) * 0.01;
            // NEW: 'Offset angle' is used as the angle step between each measurement.
            // For the delta 2A, this is 24deg/(num of measurements)
            // and there are a total of 15 frames for the full 360 degree sweep.
            let offset_angle_deg : f32 = 24.0 / (value.measurements_count() as f32);

            // 180 degrees means the 0-point is opposite the motor location, rather than on-top of the motor.
            let start_angle_deg : f32 = ((start_angle as f32) * 0.01 + 180.0) % 360.0;

            let mut m_frame = MeasurementFrame {
                rpm,
                offset_angle:offset_angle_deg,
                start_angle:start_angle_deg,
                timestamp:value.timestamp,
                measurements: vec![],
            };

            // iterate the payload data 3 bytes at a time
            let mut readings: Vec<Measurement> = value
                .data
                .iter()
                .skip(data_start + 5)
                .collect_vec()
                .chunks(3)
                .take(value.measurements_count().into())
                .enumerate()
                .map(|(i,m)| {
                    // mapping three bytes at a time into a measurement
                    let signal_quality: u8 = *m[0];
                    let distance_msb: u8 = *m[1];
                    let distance_lsb: u8 = *m[2];
                    let dist_raw = u16::from_be_bytes([distance_msb,distance_lsb]);
                    let dist_mm = (dist_raw as f32) * 0.25;

                    let angle = start_angle_deg + (i as f32) * offset_angle_deg;

                    Measurement {
                        angle,
                        signal_quality,
                        distance_mm:dist_mm,
                    }
                })
                .collect();

            m_frame.measurements.append(&mut readings);

            m_frame
        }
    }
}

// Boilerplate for constructing a new PartialFrame object
impl PartialFrame {
    pub fn new() -> Self {
        PartialFrame::default()
    }

    pub fn reset(&mut self) {
        self.data.clear(); // clear data, parts, and bytes wanted back to normal
        self.bytes_wanted = 8; // when this hits zero we are DONEZO
    }

    pub fn finished(&self) -> bool {
        self.bytes_wanted == 0 && self.crc_16_valid()
    }

    pub fn is_measurement_type(&self) -> bool {
        if self.data.len() >= 6 {
            self.data.as_slice()[5] == 0xAD
        } else {
            false
        }
    }

    pub fn is_health_type(&self) -> bool {
        if self.data.len() >= 6 {
            self.data.as_slice()[5] == 0xAE
        } else {
            false
        }
    }

    pub fn has_payload_length(&self) -> bool {
        self.data.len() >= 8
    }

    pub fn measurements_count(&self) -> u16 {
        if self.has_payload_length() {
            let pl = self.payload_length() as u16;
            return (pl - 5) / 3;
        }
        0
    }

    pub fn payload_length(&self) -> usize {
        if self.has_payload_length() {
            let msb = self.data.as_slice()[6];
            let lsb = self.data.as_slice()[7];
            return u16::from_be_bytes([msb, lsb]) as usize;
        }
        0
    }

    pub fn crc_16_valid(&self) -> bool {
        if self.bytes_wanted == 0 {
            let end = self.data.len();
            let msb = self.data.as_slice()[end-2];
            let lsb = self.data.as_slice()[end-1];
            let crc_expected = u16::from_be_bytes([msb,lsb]);

            // sum all bytes excluding the crc
            let crc_calc : u16 = self.data.iter().map(|x| *x as u16).take(end-2).sum();

            // info!("{:?}",&self.data.as_slice());
            // info!("{},{},{}",end,msb,lsb);
            // info!("{} == {}",crc_calc, crc_expected);

            return crc_expected == crc_calc;
        }
        false
    }
}

impl Display for PartialFrame {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.finished() {
            if self.is_health_type() {
                f.write_str(&format!("PartialFrame[HEALTH]({}/{})",self.bytes_wanted,self.data.len()))
            } else if self.is_measurement_type() {
                f.write_str(&format!("PartialFrame[SCAN]({}/{})",self.bytes_wanted,self.data.len()))
            } else {
                f.write_str(&format!("PartialFrame[???]({}/{})",self.bytes_wanted,self.data.len()))
            }
        } else {
            f.write_str(&format!("PartialFrame({}/{})",self.bytes_wanted,self.data.len()))
        }
    }
}

impl Default for PartialFrame {
    fn default() -> Self {
        PartialFrame {
            data: vec![],
            bytes_wanted: 8,
            bytes_written: 0,
            timestamp: get_nanos(),
        }
    }
}

impl Write for PartialFrame {
    fn write_all(&mut self, mut buf: &[u8]) -> std::io::Result<()> {
        // clear bytes written counter
        self.bytes_written = 0;

        // returns early-Ok when frame is full
        while !buf.is_empty() {
            match self.write(buf) {
                Ok(0) => return Err(Error::from_raw_os_error(105)),
                Ok(n) => buf = &buf[n..],
                Err(ref e) if e.kind() == std::io::ErrorKind::Interrupted => {}
                Err(e) => return Err(e),
            }
        }
        Ok(())
    }

    fn by_ref(&mut self) -> &mut Self
    where
        Self: Sized,
    {
        self
    }

    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let mut bytes_eaten: usize = 0;

        let bytes_available = buf.len();

        for _i in 0..bytes_available {
            // iterate over the buffer data, adding each byte and updating the state
            let d = buf[bytes_eaten];

            if self.bytes_wanted == 0 {
                break;
            }

            let accept_byte = match (self.data.len(), d) {
                (0, 0xAA) => {
                    // update timestamp on header detect
                    self.timestamp = get_nanos();
                    // header
                    true
                }
                (1, _) => {
                    // DLC
                    true
                }
                (2, _) => {
                    // DLC
                    true
                }
                (3, 0x01) => {
                    // type
                    true
                }
                (4, 0x61) => {
                    // protocol
                    true
                }
                (5, 0xAD | 0xAE) => {
                    // command type
                    true
                }
                (6, _) => {
                    // payload len
                    true
                }
                (7, _) => {
                    // payload len
                    true
                }
                (d, _) => {
                    d >= 8 
                }
            };

            if accept_byte {
                // debug!("accept_byte");
                self.data.append(&mut vec![d]);
                self.bytes_wanted -= 1;

                // now do a match to calculate payload length
                match self.has_payload_length() {
                    true => {
                        if self.data.len() == 8 {
                            // payload len plus 2 for CRC
                            self.bytes_wanted = self.payload_length() + 2;
                        }
                    }
                    false => (),
                }
            } else {
                // didn't accept this byte - but we also aren't finished.
                // trigger reset
                debug!("reset during write");
                self.reset();
            }

            bytes_eaten += 1;
            // set the bytes written thingy
            self.bytes_written += 1;
        }
        Ok(bytes_eaten)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        // todo!()
        Ok(())
    }
}

pub fn get_nanos() -> u128 {
    // get the current epoch time in nanoseconds
    SystemTime::now().duration_since(UNIX_EPOCH).expect("Time went backwards").as_nanos()
}