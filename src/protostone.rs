use crate::{byte_utils::ByteUtils, protoburn::Protoburn};
use anyhow::{anyhow, Result};
use bitcoin::Transaction;
use ordinals::{
    runestone::{message::Message, tag::Tag},
    varint, Edict, Runestone,
};
use std::collections::HashSet;

static mut PROTOCOLS: Option<HashSet<u128>> = None;

pub fn initialized_protocol_index() -> Result<()> {
    unsafe { PROTOCOLS = Some(HashSet::new()) }
    Ok(())
}

pub fn add_to_indexable_protocols(protocol_tag: u128) -> Result<()> {
    unsafe {
        if let Some(set) = PROTOCOLS.as_mut() {
            set.insert(protocol_tag);
        }
    }
    Ok(())
}

fn has_protocol(protocol_tag: &u128) -> Result<bool> {
    unsafe {
        if let Some(set) = PROTOCOLS.as_mut() {
            let contains = set.contains(protocol_tag);
            return Ok(contains);
        }
    }
    Ok(false)
}

pub struct Protostone {
    pub burn: Option<u128>,
    pub message: Vec<u128>,
    pub edicts: Option<Vec<Edict>>,
    pub refund: Option<u32>,
    pub pointer: Option<u32>,
    pub from: Option<Vec<u32>>,
}

fn varint_byte_len(input: &Vec<u8>, n: u128) -> Result<usize> {
    let mut cloned = input.clone();
    for _i in 0..n {
        let (_, size) =
            varint::decode(&cloned.as_slice()).map_err(|_| anyhow!("varint decode error"))?;
        cloned.drain(0..size);
    }

    Ok(input.len() - cloned.len())
}

impl Protostone {
    pub fn append_edicts(&mut self, edicts: Vec<Edict>) {
        self.edicts = Some(edicts);
    }

    pub fn from_bytes(tx: &Transaction, bytes: Vec<u8>) -> Result<Self> {
        let integers =
            Runestone::integers(&bytes.as_slice()).map_err(|e| anyhow!(e.to_string()))?;
        let Message {
            edicts,
            flaw,
            mut fields,
        } = Message::from_integers(tx, &integers);
        // Can either throw or not throw
        if let Some(_) = flaw {
            return Err(anyhow!("protostone flawed"));
        }

        Ok(Protostone {
            burn: Tag::Burn.take(&mut fields, |[tag]| Some(tag)),
            message: match fields.get(&<u128 as From<Tag>>::from(Tag::Message)) {
                Some(v) => v
                    .clone()
                    .try_into()
                    .map_err(|_| anyhow!("protostone flawed"))?,
                None => Vec::<u128>::new(),
            },
            refund: Tag::Refund.take(&mut fields, |[tag]| Some(tag as u32)),
            pointer: Tag::ProtoPointer.take(&mut fields, |[tag]| Some(tag as u32)),
            from: Some(
                Tag::From
                    .take_all(&mut fields)
                    .ok_or(anyhow!("could not parse from"))?
                    .into_iter()
                    .map(|v| ByteUtils::to_u32(v))
                    .collect(),
            ),
            edicts: Some(edicts),
        })
    }

    pub fn from_runestone(tx: &Transaction, runestone: &Runestone) -> Result<Vec<Self>> {
        let protostone_raw = runestone
            .proto
            .clone()
            .ok_or(anyhow!("no protostone field in runestone"))?;
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
            let (protocol_tag, size) =
                varint::decode(&protostone_bytes.as_slice()).map_err(|e| anyhow!(e.to_string()))?;
            if protocol_tag == 0 {
                break;
            }
            if size == usize::MAX {
                break;
            }
            protostone_bytes.drain(0..size);
            let (len, size) =
                varint::decode(&protostone_bytes.as_slice()).map_err(|e| anyhow!(e.to_string()))?;
            if size == usize::MAX {
                break;
            }
            protostone_bytes.drain(0..size);
            let byte_length = varint_byte_len(&protostone_bytes, len)?;
            if has_protocol(&protocol_tag)? {
                protostones.push(Protostone::from_bytes(
                    tx,
                    (&protostone_bytes[0..byte_length]).to_vec(),
                )?);
            }
            protostone_bytes.drain(0..byte_length);
        }

        Ok(protostones)
    }
}

pub trait Protostones {
    fn burns(protostones: Self) -> Result<Vec<Protoburn>>;
}

impl Protostones for Vec<Protostone> {
    fn burns(protostones: Self) -> Result<Vec<Protoburn>> {
        Ok(protostones
            .into_iter()
            .filter(|stone| stone.burn.is_some())
            .map(|stone| Protoburn {
                tag: stone.burn,
                pointer: stone.pointer,
                from: stone.from,
            })
            .collect())
    }
}
