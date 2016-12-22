macro_rules! markers {
    ($($(#[$meta:meta])* marker $name:ident = $value:expr,)*) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub enum Marker {
            $($(#[$meta])* $name,)*
        }

        impl Marker {
            pub fn from_u8(n: u8) -> Option<Marker> {
                match n {
                    $($value => Some(Marker::$name),)*
                    _ => None,
                }
            }

            pub fn to_u8(self) -> u8 {
                match self {
                    $(Marker::$name => $value,)*
                }
            }
        }
    }
}

markers! {
    marker Number = 0x00,
    marker Boolean = 0x01,
    marker String = 0x02,
    marker Object = 0x03,
    #[doc = "Unsupported, reserved for future use"]
    marker MovieClip = 0x04,
    marker Null = 0x05,
    marker Undefined = 0x06,
    marker Reference = 0x07,
    marker ECMAArray = 0x08,
    marker ObjectEnd = 0x09,
    marker StrictArray = 0x0A,
    marker Date = 0x0B,
    marker LongString = 0x0C,
    marker Unsupported = 0x0D,
    #[doc = "Unsupported, reserved for future use"]
    marker RecordSet = 0x0E,
    marker XmlDocument = 0x0F,
    marker TypedObject = 0x10,

    #[doc = "Special marker for switching from AMF0 to AMF3"]
    marker AVMPlusObject = 0x11,
}
