use byteorder::NetworkEndian;
use byteorder::ReadBytesExt;
use chrono::{DateTime, Utc};
use markers::Marker;
use std::collections::BTreeMap;
use std::io::Read;
use Value;

use self::errors::*;

#[allow(unused_doc_comment)]
pub mod errors {
    use markers::Marker;
    use std::io;
    use std::string::FromUtf8Error;

    error_chain! {
        foreign_links {
            Utf8(FromUtf8Error);
            Io(io::Error);
        }

        errors {
            BadMarker(n: u8) {
                description("Bad marker value")
                display("Bad marker value: `{}`", n)
            }

            UnexpectedMarker(marker: Marker) {
                description("Unexpected marker encountered")
                display("Unexpected marker encountered: {:?}", marker)
            }
        }
    }
}

fn read_marker<R: Read>(r: &mut R) -> Result<Marker> {
    let b = try!(r.read_u8());
    Marker::from_u8(b).ok_or(ErrorKind::BadMarker(b).into())
}

fn read_string<R: Read>(r: &mut R, size: usize) -> Result<String> {
    // Handle special case of empty string, where we don't need to spend time
    // on "reading" the value, allocating/validating things.
    if size == 0 {
        return Ok(String::new());
    }

    let mut buf = Vec::with_capacity(size);
    try!(r.read_exact(&mut buf));
    Ok(try!(String::from_utf8(buf)))
}

fn read_object_property<R: Read>(r: &mut R) -> Result<Option<(String, Value)>> {
    let size = try!(r.read_u16::<NetworkEndian>());

    if size == 0 {
        // According to the AMF0 specification, an empty string means
        // there is no further dynamic properties following the current one,
        // so we just check here that the marker object-end is present.
        match try!(read_marker(r)) {
            Marker::ObjectEnd => Ok(None),
            marker => Err(ErrorKind::UnexpectedMarker(marker).into()),
        }
    } else {
        let key = try!(read_string(r, size as usize));
        let value = try!(read_value(r));

        Ok(Some((key, value)))
    }
}

fn read_date<R: Read>(r: &mut R) -> Result<DateTime<Utc>> {
    use chrono::NaiveDateTime;

    let timestamp = try!(r.read_f64::<NetworkEndian>());
    // Unused for now
    let _timezone = try!(r.read_i16::<NetworkEndian>());

    let secs = (timestamp / 1000.0) as i64;
    let nsecs = (timestamp % 1000.0 * 1000000.0) as u32;

    let dt = NaiveDateTime::from_timestamp(secs, nsecs);
    let dt = DateTime::from_utc(dt, Utc);

    Ok(dt)
}

/// Read items with provided function `f` while the return value is `Some`.
///
/// Will return earlier with an error if an error is encountered during reading,
/// discarding the already readed items if any.
fn read_while_some<R: Read, T, F>(r: &mut R, f: F) -> Result<Vec<T>>
    where F: Fn(&mut R) -> Result<Option<T>>
{
    let mut items = Vec::new();
    loop {
        match f(r) {
            Ok(Some(item)) => items.push(item),
            Ok(None) => break,
            Err(e) => return Err(e),
        }
    }
    Ok(items)
}

/// Read items with provided function `f` exactly `count` times.
///
/// Will return earlier with an error if an error is encountered during reading,
/// including the ones caused by an "unexpected eof" because of not enough things to read,
/// discarding the already readed items if any.
fn read_array<R: Read, T, F>(r: &mut R, count: usize, f: F) -> Result<Vec<T>>
    where F: Fn(&mut R) -> Result<T>
{
    let mut items = Vec::new();
    for _ in 0..count {
        items.push(try!(f(r)));
    }
    Ok(items)
}

fn read_object<R: Read>(r: &mut R) -> Result<BTreeMap<String, Value>> {
    read_while_some(r, read_object_property)
        .map(|items| items.into_iter().collect())
}

pub fn read_value<R: Read>(r: &mut R) -> Result<Value> {
    let value = match try!(read_marker(r)) {
        Marker::Null => Value::Null,
        Marker::Undefined => Value::Undefined,
        Marker::Unsupported => Value::Unsupported,
        Marker::MovieClip => Value::Unsupported,
        Marker::RecordSet => Value::Unsupported,
        Marker::Number =>
            Value::Number(try!(r.read_f64::<NetworkEndian>())),
        Marker::Boolean =>
            Value::Boolean(try!(r.read_u8()) != 0),
        Marker::String => {
            let size = try!(r.read_u16::<NetworkEndian>());
            Value::String(try!(read_string(r, size as usize)))
        }
        Marker::LongString => {
            let size = try!(r.read_u32::<NetworkEndian>());
            Value::String(try!(read_string(r, size as usize)))
        }
        Marker::Object => Value::Object(try!(read_object(r))),
        Marker::TypedObject => {
            let name_size = try!(r.read_u16::<NetworkEndian>());
            let name = try!(read_string(r, name_size as usize));
            Value::TypedObject(name, try!(read_object(r)))
        }
        Marker::Reference =>
            Value::Reference(try!(r.read_u16::<NetworkEndian>())),
        Marker::ECMAArray => {
            let count = try!(r.read_u32::<NetworkEndian>());
            let items = try!(read_array(r, count as usize, read_object_property));
            // Remove all None from Vec and keep the others, thus remove the
            // Option<T> "layer"
            let items = items.into_iter().filter_map(|item| item);
            Value::ECMAArray(items.collect())
        }
        Marker::StrictArray => {
            let count = try!(r.read_u32::<NetworkEndian>());
            let items = try!(read_array(r, count as usize, read_value));
            Value::Array(items)
        }
        Marker::XmlDocument => {
            let size = try!(r.read_u32::<NetworkEndian>());
            Value::XmlDocument(try!(read_string(r, size as usize)))
        }
        Marker::Date => Value::Date(try!(read_date(r))),
        marker => return Err(ErrorKind::UnexpectedMarker(marker).into()),
    };

    Ok(value)
}
