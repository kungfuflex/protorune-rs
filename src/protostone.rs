use crate::byte_utils::ByteUtils;
use crate::protoburn::Protoburn;
use bitcoin::Transaction;
use metashrew::byte_view::shrink_back;
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
            Self::Encode => "error encoding protostone",
            Self::NoProtostone => "no protostone found",
            Self::Flawed => "corrupted protostonea",
            Self::VarintError(_) => "varint error"
        };
        write!(f, "{}", s)
    }
}

pub struct Protostone {
    burn: Option<u32>,
    message: Vec<u128>,
    edicts: Option<Vec<Edict>>,
    refund: Option<u32>,
    pointer: Option<u32>,
    from: Option<u32>,
}

fn varint_byte_len(input: &Vec<u8>, n: u128) -> Result<usize> {
    let mut cloned = input.clone();
    for i in 0..n {
        let (_, size) = varint::decode(&cloned.as_slice()).map_err(|e| ProtostoneError::VarintError(e))?;
        cloned.drain(0..size);
    }

    Ok(input.len() - cloned.len())
}

impl Protostone {
    pub fn append_edicts(&mut self, edicts: Vec<Edict>) {
        self.edicts = Some(edicts);
    }

    pub fn from_bytes(tx: &Transaction, bytes: Vec<u8>) -> Result<Self> {
        let integers = Runestone::integers(&bytes.as_slice()).map_err(|e| ProtostoneError::VarintError(e))?;
        let Message {
            edicts,
            flaw,
            mut fields,
        } = Message::from_integers(tx, &integers);
        // Can either throw or not throw
        if let Some(_) = flaw {
            return Err(ProtostoneError::Flawed);
        }

        Ok(Protostone {
            burn: Tag::Burn.take(&mut fields, |[tag]| Some(tag as u32)),
            message: match fields.get(&<u128 as From<Tag>>::from(Tag::Message)) {
              Some(v) => v.clone().try_into().map_err(|_| ProtostoneError::Flawed)?,
              None => Vec::<u128>::new()
            },
            refund: Tag::Refund.take(&mut fields, |[tag]| Some(tag as u32)),
            pointer: Tag::ProtoPointer.take(&mut fields, |[tag]| Some(tag as u32)),
            from: Tag::From.take(&mut fields, |[tag]| Some(tag as u32)),
            edicts: Some(edicts),
        })
    }

    pub fn from_runestone(tx: &Transaction, runestone: &Runestone) -> Result<Vec<Self>> {
        let protostone_raw = runestone.proto.clone().ok_or(ProtostoneError::NoProtostone)?;
        let protostone_raw_len = protostone_raw.len();
        let mut protostone_bytes = protostone_raw
            .into_iter()
            .enumerate()
            .map(|(i, v)| -> Vec<u8> {
                if i == protostone_raw_len - 1 {
                    <u128 as ByteUtils>::snap_to_15_bytes(v)
                } else {
                  <u128 as ByteUtils>::to_aligned_bytes(v)
                }
            })
            .flatten()
            .collect::<Vec<u8>>();
        let mut protostones: Vec<Self> = vec![];
        while protostone_bytes.len() > 0 {
            let (protocol_tag, size) = varint::decode(&protostone_bytes.as_slice()).map_err(|e| ProtostoneError::VarintError(e))?;
            if protocol_tag == 0 {
                break;
            }
            if size == usize::MAX {
                break;
            }
            protostone_bytes.drain(0..size);
            let (len, size) = varint::decode(&protostone_bytes.as_slice()).map_err(|e| ProtostoneError::VarintError(e))?;
            if size == usize::MAX {
                break;
            }
            protostone_bytes.drain(0..size);
            let byte_length = varint_byte_len(&protostone_bytes, len)?;
            protostones.push(Protostone::from_bytes(tx, (&protostone_bytes[0..byte_length]).to_vec())?);
            protostone_bytes.drain(0..byte_length);
        }

        Ok(protostones)
    }
}
