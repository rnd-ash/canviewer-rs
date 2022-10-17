

#[derive(Debug, Clone, PartialEq, PartialOrd, Default)]
pub struct Signal {
    pub name: String,
    pub comment: Option<String>,
    pub signal_type: SignalType,
    pub order: ByteOrder,
    pub start_bit: u64,
    pub length_bits: u64,
    pub unit: String,
    pub min: f32,
    pub max: f32,
    pub signed: bool,
} 

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum ByteOrder {
    LittleEndian,
    BigEndian
}

impl Default for ByteOrder {
    fn default() -> Self {
        Self::LittleEndian
    }
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum SignalType {
    Bool,
    Linear { multi: f32, offset: f32 },
    Enum(Vec<(i64, String)>)
}

impl Default for SignalType {
    fn default() -> Self {
        Self::Linear { multi: 1.0, offset: 0.0 }
    }
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Default)]
pub struct Message {
    pub id: u32,
    pub name: String,
    pub comment: Option<String>,
    pub length_bytes: u64,
    pub signals: Vec<Signal>,
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Default)]
pub struct Ecu {
    pub name: String,
    pub messages: Vec<Message>
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Default)]
pub struct TreeDbc {
    pub ecus: Vec<Ecu>
}