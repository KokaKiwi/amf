use byteorder::NetworkEndian;
use byteorder::ReadBytesExt;
use markers::Marker;
use std::io::Read;
use Value;

use self::errors::*;

pub mod errors {
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
        }
    }
}

fn read_marker<R: Read>(r: &mut R) -> Result<Marker> {
    let b = try!(r.read_u8());
    Marker::from_u8(b).ok_or(ErrorKind::BadMarker(b).into())
}

fn read_bool<R: Read>(r: &mut R) -> Result<bool> {
    let b = try!(r.read_u8());
    Ok(b != 0)
}

fn read_number<R: Read>(r: &mut R) -> Result<f64> {
    Ok(try!(r.read_f64::<NetworkEndian>()))
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

pub fn read_value<R: Read>(r: &mut R) -> Result<Value> {
    let value = match try!(read_marker(r)) {
        Marker::Null => Value::Null,
        Marker::Undefined => Value::Undefined,
        Marker::Unsupported => Value::Unsupported,
        Marker::Number => Value::Number(try!(read_number(r))),
        Marker::Boolean => Value::Boolean(try!(read_bool(r))),
        Marker::String => {
            let size = try!(r.read_u16::<NetworkEndian>());
            Value::String(try!(read_string(r, size as usize)))
        }
        Marker::LongString => {
            let size = try!(r.read_u32::<NetworkEndian>());
            Value::String(try!(read_string(r, size as usize)))
        }
        _ => unimplemented!(),
    };

    Ok(value)
}
