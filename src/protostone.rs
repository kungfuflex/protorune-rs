use crate::byte_utils::ByteUtils;
use crate::protoburn::Protoburn;
use bitcoin::Transaction;
use metashrew_rs::byte_view::shrink_back;
use ordinals::{
    runestone::{message::Message, tag::Tag},
    varint, Edict, Runestone,
};
use std::fmt;

enum ProtostoneError {
    Encode,
    NoProtostone,
    VarintError(varint::Error),
    Flawed,
}

type Result<T> = std::result::Result<T, ProtostoneError>;

impl fmt::Display for ProtostoneError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::EncodeError => "Error encoding protostone",
            Self::NoProtostoneError => "No protostone found",
            Self::Flawed => "Corrupted protostonea",
        };
        write!(f, s)
    }
}

impl Bytes for u128 {}

pub struct Protostone {
    burn: Option<u32>,
    message: Option<u32>,
    edicts: Option<Vec<Edict>>,
    refund: Option<u32>,
    pointer: Option<u32>,
    from: Option<u32>,
}

fn varint_byte_len(input: Vec<u8>, n: u128) -> Result<usize> {
    let mut cloned = input.clone();
    for i in 0..n {
        let (_, size) = varint::decode(&cloned.as_slice())?;
        cloned.drain(0..size);
    }

    Ok(input.len() - cloned.len())
}

impl Protostone {
    pub fn append_edicts(&mut self, edicts: Vec<Edict>) -> Self {
        self.edicts = Some(edicts);
        self
    }

    pub fn from_bytes(tx: &Transaction, bytes: Vec<u8>) -> Result<Self> {
        let integers = Runestone::integers(&bytes.as_slice())?;
        let Message {
            edicts,
            flaw,
            mut fields,
        } = Message::from_integers(tx, bytes);
        // Can either throw or not throw
        if Some(flaw) {
            Err(ProtostoneError::Flawed)
        }

        Ok(Protostone {
            burn: Tag::Burn.take(&mut fields, |[tag]| Some(tag)),
            message: Tag::Message.take(&mut fields, |[tag]| Some(tag)),
            refund: Tag::Refund.take(&mut fields, |[tag]| Some(tag)),
            pointer: Tag::ProtoPointer.take(&mut fields, |[tag]| Some(tag)),
            from: Tag::From.take(&mut fields, |[tag]| Some(tag)),
            edicts: Some(edicts),
        })
    }

    pub fn from_runestone(tx: &Transaction, runestone: &Runestone) -> Result<Vec<Self>> {
        let protostone_raw = runestone.proto.ok_or(ProtostoneError::NoProtostone)?;
        let mut protostone_bytes = protostone_raw
            .into_iter()
            .enumerate()
            .map(|(i, v)| {
                if i == protostone_raw.len() - 1 {
                    ByteUtils::snap_to_15_bytes(v)
                }
                ByteUtils::to_aligned_bytes(v)
            })
            .flatten()
            .collect::<Vec<u8>>();
        let mut protostones: Vec<Self> = vec![];
        while protostone_bytes.len() > 0 {
            let (protocol_tag, size) = varint::decode(&protostone_bytes.as_slice())?;
            if protocol_tag == 0 {
                break;
            }
            if size == u128::MAX {
                break;
            }
            protostone_bytes.drain(0..size);
            let (len, size) = varint::decode(&protostone_bytes.as_slice())?;
            if size == u128::MAX {
                break;
            }
            protostone_bytes.drain(0..size);
            let byte_length = varint_byte_len(input, len);
            protostones.push(Protostone::from_bytes(protostone_bytes[0..byte_length]));
            protostone_bytes.drain(0..byte_length);
        }

        Ok(protostones)
    }
}
