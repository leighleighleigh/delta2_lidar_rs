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

// This type+enum is used to keep track of what bytes mean what, and how many bytes each
// part of the frame is. I've made a simple tuple type to represent this, hopefully it will make 
// the state machine code much cleaner! (no idea tho).
// offset, length, expected constnat value
use log::{info,warn,debug};
use std::{io::{Write,Error}, fmt::Display};
use itertools::Itertools;

#[derive(Debug,Clone,Copy,PartialEq, Eq)]
pub struct FramePart {
    pub offset : i32,
    pub length : i32,
    pub expected : Option<u8>,
}

// These are the hard-coded 'magic numbers' which we expect to receive in each frame.
const FRAME_HEADER : FramePart = FramePart{ offset: 0, length: 1, expected: Some(0xAA) };
const FRAME_LENGTH : FramePart = FramePart{ offset: 1, length: 2, expected: None };
const FRAME_VERSION : FramePart = FramePart{ offset: 3, length: 1, expected: Some(0x01) };
const FRAME_TYPE : FramePart = FramePart{ offset: 4, length: 1, expected: Some(0x61) };
const FRAME_CMD : FramePart = FramePart{ offset: 5, length: 1, expected: None };
const FRAME_CMD_DEVICEHEALTH : FramePart = FramePart{ offset: 5, length: 1, expected: Some(0xAE) };
const FRAME_CMD_MEASUREMENT : FramePart = FramePart{ offset: 5, length: 1, expected: Some(0xAD) };
const FRAME_PAYLOAD_LENGTH : FramePart = FramePart{ offset: 6, length: 2, expected: None };
// starts at byte 8, ends at LEN-3.
const FRAME_PAYLOAD : FramePart = FramePart{ offset: 8, length: -3, expected: None };
// starts at LEN-2, ends at LEN.
const FRAME_CRC : FramePart = FramePart{ offset: -2, length: 2, expected: None };


#[derive(Debug,Clone)]
pub struct PartialFrame {
    pub data : Vec<u8>, // buffered input data gets copied here, to be decoded at the end
    pub parts : Vec<FramePart>, // completed parts of the frame are added here
    pub bytes_wanted : usize, // how many bytes to read until we complete the next part
}

// Boilerplate for constructing a new PartialFrame object
impl PartialFrame {
    pub fn new() -> Self {
        PartialFrame::default()
    }

    pub fn reset(&mut self) {
        // clear data, parts, and bytes wanted back to normal
        self.data.clear();
        self.parts.clear();
        self.bytes_wanted = 1; 
    }

    pub fn build(&mut self) -> Result<Option<Self>,Error> {
        if self.finished() {
            return Ok(Some(self.clone()))
        }

        if self.part_bytes_remaining() == 0 {

            let part = self.next();
            debug!("Finished: {}",part);
            self.parts = [self.parts.clone(), vec![part.clone()]].concat();

            // get next part
            let next_part = self.next();

            if next_part == &FRAME_PAYLOAD {
                debug!("GOT PAYLOAD LENGTH PART");
                // extract the payload length, using FRAME_PAYLOAD_LENGTH to define the byte ranges.
                let payload_len_offset = FRAME_PAYLOAD_LENGTH.offset as usize;

                let mut payload_len_bytes : [u8;2] = [0,0];

                // push bytes on. basically hard-code 2 here
                for (i,b) in self.data.clone().iter().skip(payload_len_offset-1).enumerate().take(2) {
                    payload_len_bytes[i] = *b;
                }

                debug!("RAW[{}]: {:?}",self.data.len(),self.data.as_slice());
                debug!("LEN: {:?}",payload_len_bytes);
                let payload_len = u16::from_le_bytes(payload_len_bytes);
                debug!("payload length: {}",payload_len);

                self.bytes_wanted = payload_len as usize;
            } else {
                self.bytes_wanted = next_part.length as usize;
            }
        }

        Ok(None)
    }

    pub fn part_bytes_remaining(&self) -> usize {
        self.bytes_wanted 
    }

    pub fn finished_parts(&self) -> &Vec<FramePart> {
        &self.parts
    }

    pub fn last(&self) -> &FramePart {
        match self.parts.last() {
            Some(p) => p,
            None => &FRAME_HEADER,
        }
    }

    pub fn next(&self) -> &FramePart {
        let part = self.last();

        let next_part = match part {
            &FRAME_HEADER => &FRAME_LENGTH,
            &FRAME_LENGTH => &FRAME_VERSION,
            &FRAME_VERSION => &FRAME_TYPE,
            &FRAME_TYPE => &FRAME_CMD,
            &FRAME_CMD=> &FRAME_PAYLOAD_LENGTH,
            &FRAME_CMD_DEVICEHEALTH => &FRAME_PAYLOAD_LENGTH,
            &FRAME_CMD_MEASUREMENT => &FRAME_PAYLOAD_LENGTH,
            &FRAME_PAYLOAD_LENGTH => &FRAME_PAYLOAD,
            &FRAME_PAYLOAD => &FRAME_CRC,
            &FRAME_CRC => &FRAME_CRC,
            _ => {
                panic!("BRUH");
            }
        };

        next_part
    }

    pub fn finished(&self) -> bool {
        self.part_bytes_remaining() == 0 && self.next() == &FRAME_CRC && self.last() == &FRAME_CRC
    }

    pub fn check_expected_data(&mut self) {
        if self.part_bytes_remaining() == 0 {
            let end = match self.data.clone().last() {
                Some(e) => {
                    *e
                }
                None => {
                    return;
                }
            };

            let cmd_meas = FRAME_CMD_MEASUREMENT.expected.unwrap();
            let cmd_health = FRAME_CMD_DEVICEHEALTH.expected.unwrap();

            // if the next value is CMD type, we will allow two different values
            match self.next() == &FRAME_CMD {
                true => {
                    match (end == cmd_meas || end == cmd_health) {
                        true => {
                            if end == cmd_meas {
                                debug!("MEASUREMENT: {} == {}",end,cmd_meas);
                            } else {
                                debug!("HEALTH: {} == {}",end,cmd_health);
                            }
                        },
                        false => {
                            debug!("RESET: {} != {} or {}",end,cmd_meas,cmd_health);
                            self.reset();
                        }
                    }
                },
                false => {
                    match self.next().expected {
                        Some(exp) => {
                            if exp != end {
                                debug!("RESET: {} != {}",exp,end);
                                self.reset();
                            } else {
                                debug!("OKAY: {} == {}",exp,end);
                            }
                        },
                        None => {}
                    }
                },
            }
        }
    }
}


impl Display for FramePart {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!("[{}:{}]",self.offset,self.length))
    }
}


impl Display for PartialFrame {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // write out the length of data we have, the part bytes remaining, and the last/next part
        match self.finished() {
            true => {
                f.write_str(&format!("Frame(got: {},want: {},parts: {})",self.data.len(),self.part_bytes_remaining(),self.finished_parts().iter().format(",")))
            },
            false => {
                f.write_str(&format!("PartialFrame(got: {},want: {},parts: {})",self.data.len(),self.part_bytes_remaining(),self.finished_parts().iter().format(",")))
            },
        }
    }
}


impl Default for PartialFrame {
    fn default() -> Self {
        PartialFrame { data: vec![], parts: vec![], bytes_wanted: 1 }
    }
}

impl Write for PartialFrame {
    
    fn write_all(&mut self, mut buf: &[u8]) -> std::io::Result<()> {
        while !buf.is_empty() {
            match self.write(buf) {
                Ok(0) => {
                    return Err(Error::from_raw_os_error(105))
                }
                Ok(n) => {
                    // decrement bytes wanted
                    // info!("bytes_wanted = {}",self.bytes_wanted);
                    // info!("n = {}",n);
                    self.bytes_wanted -= n;
                    self.check_expected_data();

                    match self.build() {
                        Ok(done) => {
                            if done.is_some() {
                                info!("{}",done.unwrap());
                            }
                        },
                        Err(e) => {
                            warn!("{}",e)
                        }
                    }
                    // warn!("POST[{}], {:?}",self.data.len(), self.data.last());
                    // debug!("RAW[{}]: {:?}",self.data.len(),self.data.as_slice());

                    buf = &buf[n..]
                },
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
        debug!("write buf: {:?}", buf);
        // take only bytes wanted from the buffer
        let want : usize = match buf.len() > self.part_bytes_remaining() {
            true => {
                self.part_bytes_remaining()
            },
            false => {
                buf.len()
            },
        };

        if want == 0 {
            return Ok(0);
        }

        // internal buffer is allowed to grow unbounded
        let res:Vec<u8> = [self.data.as_slice(), &buf[0..want]].concat();
        // self.data = [self.data.clone(), buf[0..want].to_vec().clone()].concat();
        self.data = res.clone();

        Ok(want)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        // todo!()
        Ok(())
    }
}