use chrono::{DateTime, UTC};
use std::collections::BTreeMap;

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
