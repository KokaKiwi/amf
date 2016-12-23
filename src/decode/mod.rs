use byteorder::ReadBytesExt;
use markers::Marker;
use std::io::Read;
use Value;

use self::errors::*;

pub mod errors {
    error_chain! {
        foreign_links {
            Io(::std::io::Error);
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

pub fn read_value<R: Read>(r: &mut R) -> Result<Value> {
    let value = match try!(read_marker(r)) {
        Marker::Null => Value::Null,
        Marker::Undefined => Value::Undefined,
        Marker::Unsupported => Value::Unsupported,
        Marker::Boolean => Value::Boolean(try!(read_bool(r))),
        _ => unimplemented!(),
    };

    Ok(value)
}