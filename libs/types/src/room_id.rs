use std::fmt;

use rand::seq::IndexedRandom as _;
use schemars::{JsonSchema, json_schema};
use serde::{Deserialize, Serialize, de};

const ROOM_ID_LENGTH: usize = 8;
const ROOM_ID_CHARACTERS: &'static [u8] = concat!(
    "ABCDEFGHIJKLMNOPQRSTUVWXYZ",
    "abcdefghijklmnopqrstuvwxyz",
    "0123456789",
    "_"
)
.as_bytes();

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RoomId([u8; ROOM_ID_LENGTH]);

impl RoomId {
    pub fn new() -> Self {
        let mut thread_rng = rand::rng();
        let mut v = Vec::new();
        v.resize_with(ROOM_ID_LENGTH, || {
            *ROOM_ID_CHARACTERS.choose(&mut thread_rng).unwrap()
        });
        Self(v.try_into().unwrap())
    }
}

#[derive(Debug)]
pub enum TryParseRoomIdError {
    BadChar,
    BadLength,
}

impl fmt::Display for TryParseRoomIdError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::BadChar => write!(f, "bad character(s)"),
            Self::BadLength => write!(f, "bad length"),
        }
    }
}

impl TryFrom<&str> for RoomId {
    type Error = TryParseRoomIdError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        if value.len() != ROOM_ID_LENGTH {
            return Err(TryParseRoomIdError::BadLength);
        } else if value.bytes().any(|b| !ROOM_ID_CHARACTERS.contains(&b)) {
            return Err(TryParseRoomIdError::BadChar);
        }
        Ok(Self(value.as_bytes().try_into().unwrap()))
    }
}

impl Serialize for RoomId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(unsafe { std::str::from_utf8_unchecked(&self.0) })
    }
}

impl<'de> Deserialize<'de> for RoomId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        if deserializer.is_human_readable() {
            let s = String::deserialize(deserializer)?;
            RoomId::try_from(s.as_str()).map_err(de::Error::custom)
        } else {
            // TODO: should I really implement this...?
            unimplemented!()
        }
    }
}

impl JsonSchema for RoomId {
    fn inline_schema() -> bool {
        true
    }

    fn schema_name() -> std::borrow::Cow<'static, str> {
        "string".into()
    }

    fn json_schema(_: &mut schemars::SchemaGenerator) -> schemars::Schema {
        json_schema!({
            "type": "string"
        })
    }
}
