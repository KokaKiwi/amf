#![recursion_limit = "1024"]
extern crate byteorder;
extern crate chrono;
#[macro_use] extern crate error_chain;

use chrono::{DateTime, UTC};
use std::collections::BTreeMap;

pub mod decode;
pub mod encode;
mod markers;

pub enum Value {
    Null,
    Undefined,
    Unsupported,
    Number(f64),
    Boolean(bool),
    /// Represent an AMF string.
    ///
    /// Will be encoded/decoded as "normal"-length string or "long"-length string
    /// as needed during reading/writing.
    String(String),
    ECMAArray(BTreeMap<String, Value>),
    Array(Vec<Value>),
    Object(BTreeMap<String, Value>),
    TypedObject(String, BTreeMap<String, Value>),
    Date(DateTime<UTC>),
    XmlDocument(String),
    Reference(u16),
}
