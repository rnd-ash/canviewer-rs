use std::fmt::format;

use bitreader::BitReader;

use crate::tree_dbc::Signal;


pub type SignalParseResult<T> = std::result::Result<T, SignalParseError>;

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum ParsedSignal {
    Enum(i64, String),
    Bool(bool),
    Number(f32, Option<String>)
}

impl ToString for ParsedSignal {
    fn to_string(&self) -> String {
        match self {
            ParsedSignal::Enum(r,e) => format!("{} ({})", r, e),
            ParsedSignal::Bool(b) => format!("{}", b),
            ParsedSignal::Number(raw, unit) => {
                if let Some(u) = &unit {
                    format!("{} {}", raw, u)
                } else {
                    format!("{}", raw)
                }
            },
        }
    }
}

#[derive(Debug, Clone)]
pub enum SignalParseError {
    RangeTooBig
}

pub fn parse_signal(signal: &Signal, raw: &[u8]) -> SignalParseResult<ParsedSignal> {
    if signal.length_bits+signal.start_bit > (raw.len()*8) as u64 {
        return Err(SignalParseError::RangeTooBig)
    }

    let mut r = BitReader::new(raw);
    
    let _ignored = r.skip(signal.start_bit);
    let data = match signal.signed {
        true => r.read_i64(signal.length_bits as u8).unwrap(),
        false => r.read_u64(signal.length_bits as u8).unwrap() as i64
    };

    // Now process signal
    match &signal.signal_type {
        crate::tree_dbc::SignalType::Bool => {
            Ok(ParsedSignal::Bool(data != 0))
        },
        crate::tree_dbc::SignalType::Linear { multi, offset } => {
            Ok(ParsedSignal::Number(((data as f32)*multi) + offset, Some(signal.unit.clone())))
        },
        crate::tree_dbc::SignalType::Enum(entries) => {
            for x in entries {
                if x.0 == data {
                    return Ok(ParsedSignal::Enum(data as i64, x.1.clone()))
                }
            }
            Ok(ParsedSignal::Enum(data as i64, "INVALID VALUE".into()))
        },
    }
}