use std::time::Instant;
use std::fmt;
use std::num::ParseIntError;
use std::cmp::Ordering;
use std::panic::resume_unwind;
use libc;
use super::hex;

use std::time::{SystemTime, UNIX_EPOCH};
use std::ptr::{null, null_mut};
use std::os::raw::c_uint;

#[derive(Debug, Clone, Eq)]
pub struct ObjectId {
    pub timestamp: i32,
    pub counter:   i64,
}

impl ObjectId {

    fn from_hex(data: &str) -> Result<ObjectId, ParseError> {
        let bytes = match hex::decode(data) {
            Ok(result) => result,
            Err(_) => return Err(ParseError::Length())
        };

        if bytes.len() != 12 {
            return Err(ParseError::Length());
        }

        let mut timestamp_buffer: [u8; 4] = [0; 4];
        timestamp_buffer.copy_from_slice(&bytes[0..4]);
        let timestamp = i32::from_be_bytes(timestamp_buffer);

        let mut counter_buffer: [u8; 8] = [0; 8];
        counter_buffer.copy_from_slice(&bytes[4..12]);
        let counter = i64::from_be_bytes(counter_buffer);

        Ok(ObjectId { timestamp, counter })
    }

    fn to_hex(&self) -> String {
        let mut result = vec![];
        result.resize(12, 0);
        let timestamp_le: [u8; 4] = self.timestamp.to_be_bytes();
        let counter_le: [u8; 8] = self.counter.to_be_bytes();

        result[0..4].copy_from_slice(&timestamp_le);
        result[4..12].copy_from_slice(&counter_le);

        hex::encode(result)
    }

}

impl fmt::Display for ObjectId {

    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.to_hex().as_str())
    }

}

impl Ord for ObjectId {

    fn cmp(&self, other: &Self) -> Ordering {
        let tmp = self.timestamp.cmp(&other.timestamp);
        match tmp {
            Ordering::Equal => self.counter.cmp(&other.counter),
            _  => tmp
        }
    }

}

impl PartialOrd for ObjectId {

    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }

}

impl PartialEq for ObjectId {

    fn eq(&self, other: &Self) -> bool {
        self.counter == other.counter && self.timestamp == other.timestamp
    }

}

#[derive(Debug)]
pub struct ObjectIdMaker {
    pub counter:   i64,
}

#[derive(Debug, Clone)]
pub enum ParseError {
    ParseInt(ParseIntError),
    Length(),
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ParseError::ParseInt(parse_int_err) => parse_int_err.fmt(f),
            ParseError::Length() =>
                write!(f, "the length of str should be 12")
        }
    }
}

impl std::error::Error for ParseError{}

impl From<ParseIntError> for ParseError {
    fn from(err: ParseIntError) -> ParseError {
        return ParseError::ParseInt(err);
    }
}

fn random_i32() -> i32 {
    unsafe {
        libc::rand()
    }
}

fn random_counter() -> i64 {
    let i1: i64 = random_i32() as i64;
    let i2: i64 = random_i32() as i64;
    i1 << 32 | i2
}

impl ObjectIdMaker {

    pub fn new() -> ObjectIdMaker {
        unsafe {
            let time = libc::time(null_mut());
            libc::srand(time as c_uint);
        }
        let counter: i64 = random_counter();
        return ObjectIdMaker { counter };
    }

    pub fn mk_object_id(&mut self) -> ObjectId {
        let start = SystemTime::now();
        let since_the_epoch = start
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards");

        let in_ms = since_the_epoch.as_secs() * 1000 +
            since_the_epoch.subsec_nanos() as u64 / 1_000_000;

        let id = self.counter;
        self.counter += 1;
        ObjectId {
            timestamp: in_ms as i32,
            counter : id,
        }
    }

    pub fn value_of(content: &str) -> Result<ObjectId, ParseError> {
        if content.len() != 12 {
            return Err(ParseError::Length())
        }

        let timestamp_str = &content[0..4];
        let counter_str = &content[4..12];

        let timestamp: i32 = timestamp_str.parse::<i32>()?;
        let counter: i64 = counter_str.parse::<i64>()?;

        Ok(ObjectId {
            timestamp,
            counter,
        })
    }

}

#[cfg(test)]
mod tests {
    use crate::bson::object_id::{ ObjectIdMaker, ObjectId };

    #[test]
    fn object_id_not_zero() {
        let mut maker = ObjectIdMaker::new();
        let oid = maker.mk_object_id();

        assert_ne!(oid.timestamp, 0);
    }

    #[test]
    fn object_to_hex() {
        let mut maker = ObjectIdMaker::new();
        let oid = maker.mk_object_id();

        let hex_str = oid.to_hex();
        let from_hex = ObjectId::from_hex(hex_str.as_str()).expect("parse error");

        assert_eq!(from_hex, oid)
    }

}
