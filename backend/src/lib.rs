use can_dbc::Signal;

use can_dbc::*;
pub use tree_dbc::TreeDbc;
pub mod tree_dbc;
pub type CanResult<T> = Result<T, CanViewError>;
pub mod signal_parser;
pub use signal_parser::*;


#[derive(Debug, Clone)]
pub enum CanViewError {
    DbcError(String),
    SignalParseError(String)
}

impl ToString for CanViewError {
    fn to_string(&self) -> String {
        format!("{:?}", self)
    }
}

impl<'a> From<can_dbc::Error<'a>> for CanViewError {
    fn from(e: can_dbc::Error<'a>) -> Self {
        match e {
            can_dbc::Error::Incomplete(curr_dbc, left_over) => {
                Self::DbcError("Incomplete DBC!".into())
            },
            can_dbc::Error::Nom(nom_err) => {
                Self::DbcError(format!("DBC Parse nom error: {}", nom_err.to_string()))
            },
            can_dbc::Error::MultipleMultiplexors => {
                Self::DbcError("Multiple identical Multiplexors exist in DBC!".into())
            },
        }
    }
}

fn locate_signal_comment(dbc: &DBC, msg_id: &MessageId, sig_name: &String) -> Option<String> {
    for c in dbc.comments() {
        if let Comment::Signal { message_id, signal_name, comment } = c {
            if message_id == msg_id && signal_name == sig_name {
                return Some(comment.clone())
            }
        }
    }
    return None;
}

fn locate_message_comment(dbc: &DBC, msg_id: &MessageId) -> Option<String> {
    for c in dbc.comments() {
        if let Comment::Message { message_id, comment } = c {
            if message_id == msg_id {
                return Some(comment.clone())
            }
        }
    }
    return None;
}

fn get_signal_type(dbc: &DBC, message: &MessageId, signal: &Signal) -> tree_dbc::SignalType {
    if signal.signal_size == 1 {
        return tree_dbc::SignalType::Bool
    }
    // Iterate over all value descriptions
    let m = message.clone();
    let v = dbc.value_descriptions_for_signal(m, signal.name());
    if let Some(enum_entries) = v {
        let mut x: Vec<(i64, String)> = Vec::new();
        for e in enum_entries {
            x.push((*e.a() as i64, e.b().to_string()))
        }
        x.sort_by(|e,f| e.0.partial_cmp(&f.0).unwrap());

        return tree_dbc::SignalType::Enum(x)
    } else {
        return tree_dbc::SignalType::Linear { multi: signal.factor as f32, offset: signal.offset as f32 }
    }
}

pub fn load_dbc_from_bytes(b: &[u8]) -> CanResult<TreeDbc> {
    let dbc = can_dbc::DBC::from_slice(b)?;
    let mut finished_dbc = tree_dbc::TreeDbc::default();
    for message in dbc.messages() {
        let mut signal_array = Vec::new();
        for signal in message.signals() {
            let sig = tree_dbc::Signal {
                name: signal.name().clone(),
                comment: locate_signal_comment(&dbc, message.message_id(), signal.name()),
                signal_type: get_signal_type(&dbc, message.message_id(), signal),
                order: match signal.byte_order() {
                    ByteOrder::LittleEndian => tree_dbc::ByteOrder::LittleEndian,
                    ByteOrder::BigEndian => tree_dbc::ByteOrder::BigEndian,
                },
                start_bit: signal.start_bit,
                length_bits: signal.signal_size,
                unit: signal.unit().clone(),
                min: signal.min as f32,
                max: signal.max as f32,
                signed: *signal.value_type() == ValueType::Signed
            };
            signal_array.push(sig);
        }
        // Done with all signals
        let msg = tree_dbc::Message {
            id: message.message_id().0,
            name: message.message_name().clone(),
            comment: locate_message_comment(&dbc, message.message_id()),
            length_bytes: *message.message_size(),
            signals: signal_array,
        };
        let sender_name: String = match message.transmitter() {
            Transmitter::NodeName(name) => name.clone(),
            Transmitter::VectorXXX => "NULL SENDER".to_string(),
        };
        let mut add_ecu = true;
        for ecu in finished_dbc.ecus.iter_mut() {
            if ecu.name == sender_name {
                add_ecu = false;
                ecu.messages.push(msg.clone());
                break;
            }
        }
        if add_ecu {
            let ecu = tree_dbc::Ecu {
                name: sender_name,
                messages: vec![msg]
            };
            finished_dbc.ecus.push(ecu);
        }
    }
    Ok(finished_dbc)
}
