use crate::{
    balance_sheet::BalanceSheet,
    byte_utils::ByteUtils,
    message::{MessageContext, MessageContextParcel},
    protoburn::{Protoburn, Protoburns},
    rune_transfer::{OutgoingRunes, RuneTransfer},
    tables::RuneTable,
};
use anyhow::{anyhow, Result};
use bitcoin::{Block, OutPoint, Transaction, Txid};
use metashrew::index_pointer::{AtomicPointer, IndexPointer, KeyValuePointer};
use ordinals::{
    runestone::{message::Message, tag::Tag},
    varint, Edict, Runestone,
};
use std::collections::{HashMap, HashSet};

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

#[derive(Clone)]
pub struct Protostone {
    pub burn: Option<u128>,
    pub message: Vec<u128>,
    pub edicts: Option<Vec<Edict>>,
    pub refund: Option<u32>,
    pub pointer: Option<u32>,
    pub from: Option<Vec<u32>>,
    pub protocol_tag: u128,
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
    pub fn is_message(&self) -> bool {
        !self.message.is_empty()
    }

    pub fn process_message<T: MessageContext>(
        &self,
        atomic: &mut AtomicPointer,
        transaction: &Transaction,
        txindex: u32,
        block: &Block,
        height: u64,
        runestone_output_index: u32,
        vout: u32,
        balances_by_output: &mut HashMap<u32, BalanceSheet>,
        default_output: u32,
    ) -> Result<()> {
        if self.is_message() {
            let initial_sheet = balances_by_output
                .get(&vout)
                .map(|v| v.clone())
                .unwrap_or_else(|| BalanceSheet::default());
            atomic.checkpoint();
            let parcel = MessageContextParcel {
                atomic: atomic.derive(&IndexPointer::default()),
                runes: RuneTransfer::from_balance_sheet(
                    initial_sheet.clone(),
                    self.protocol_tag,
                    &mut atomic.derive(&IndexPointer::default()),
                ),
                transaction: transaction.clone(),
                block: block.clone(),
                height,
                pointer: self.pointer.unwrap_or_else(|| default_output),
                refund_pointer: self.pointer.unwrap_or_else(|| default_output),
                calldata: self
                    .message
                    .iter()
                    .map(|v| v.to_be_bytes())
                    .flatten()
                    .collect::<Vec<u8>>(),
                txindex,
                runtime_balances: Box::new(BalanceSheet::default()),
                sheets: Box::new(BalanceSheet::default()),
            };
            let pointer = self.pointer.unwrap_or_else(|| default_output);
            let refund_pointer = self.refund.unwrap_or_else(|| default_output);
            match T::handle(&parcel) {
                Ok(values) => {
                    match values.reconcile(balances_by_output, vout, pointer, refund_pointer) {
                        Ok(_) => atomic.commit(),
                        Err(_) => {
                            let sheet = balances_by_output
                                .get(&vout)
                                .map(|v| v.clone())
                                .unwrap_or_else(|| BalanceSheet::default());
                            balances_by_output.remove(&vout);
                            if !balances_by_output.contains_key(&refund_pointer) {
                                balances_by_output.insert(refund_pointer, BalanceSheet::default());
                                sheet.pipe(balances_by_output.get_mut(&refund_pointer).unwrap());
                                atomic.rollback()
                            }
                        }
                    }
                }
                Err(_) => {
                    atomic.rollback();
                }
            }
        }
        Ok(())
    }
    pub fn from_bytes(num_outputs: u32, protocol_tag: u128, bytes: Vec<u8>) -> Result<Self> {
        let integers =
            Runestone::integers(&bytes.as_slice()).map_err(|e| anyhow!(e.to_string()))?;
        let Message {
            edicts,
            flaw,
            mut fields,
        } = Message::from_integers(
            num_outputs,
            &integers,
            false, // protostone edicts can have outputs > num outputs
        );
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
            protocol_tag,
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

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut payload = Vec::new();

        if let Some(v) = self.burn {
            Tag::Burn.encode([v], &mut payload);
        }

        for m in &self.message {
            Tag::Message.encode([*m], &mut payload);
        }

        Tag::Refund.encode_option(self.refund, &mut payload);

        Tag::ProtoPointer.encode_option(self.pointer, &mut payload);

        Tag::ProtoPointer.encode_option(self.pointer, &mut payload);

        // TODO: finish

        payload
    }

    pub fn protostones_to_vec_u128(protostones: Vec<Protostone>) -> Vec<u128> {
        vec![]
    }

    pub fn from_runestone(tx: &Transaction, runestone: &Runestone) -> Result<Vec<Self>> {
        if let None = runestone.proto.as_ref() {
            return Ok(vec![]);
        }
        let protostone_raw = runestone
            .proto
            .clone()
            .ok_or(anyhow!("no protostone field in runestone"))?;

        Protostone::from_vec_u128(&protostone_raw, u32::try_from(tx.output.len()).unwrap())
    }

    /// Gets a vector of Protostones from an arbituary vector of bytes
    ///
    /// protostone_raw: LEB encoded Protostone
    /// num_outputs: needed to check that the edicts of the protostone do not exceed the
    pub fn from_vec_u128(protostone_raw: &Vec<u128>, num_outputs: u32) -> Result<Vec<Self>> {
        let protostone_raw_len = protostone_raw.len();
        let mut protostone_bytes = protostone_raw
            .into_iter()
            .enumerate()
            .map(|(i, v)| -> Vec<u8> {
                if i == protostone_raw_len - 1 {
                    <u128 as ByteUtils>::snap_to_15_bytes(*v)
                } else {
                    <u128 as ByteUtils>::to_aligned_bytes(*v)
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
                    num_outputs,
                    protocol_tag,
                    (&protostone_bytes[0..byte_length]).to_vec(),
                )?);
            }
            protostone_bytes.drain(0..byte_length);
        }

        Ok(protostones)
    }

    // when encoding a Protostone into the first layer of LEB encoding, we need to make sure it only uses the first
}

pub trait Protostones {
    fn burns(&self) -> Result<Vec<Protoburn>>;
    fn process_burns(
        &self,
        runestone: &Runestone,
        runestone_output_index: u32,
        balances_by_output: &HashMap<u32, BalanceSheet>,
        default_output: u32,
        txid: Txid,
    ) -> Result<()>;
}

impl Protostones for Vec<Protostone> {
    fn burns(&self) -> Result<Vec<Protoburn>> {
        Ok(self
            .into_iter()
            .filter(|stone| stone.burn.is_some())
            .map(|stone| Protoburn {
                tag: stone.burn,
                pointer: stone.pointer,
                from: stone.from.clone(),
            })
            .collect())
    }
    fn process_burns(
        &self,
        runestone: &Runestone,
        runestone_output_index: u32,
        balances_by_output: &HashMap<u32, BalanceSheet>,
        default_output: u32,
        txid: Txid,
    ) -> Result<()> {
        let mut burns = self.burns()?;
        burns.process(
            runestone,
            runestone_output_index,
            balances_by_output,
            default_output,
            txid,
        )?;
        Ok(())
    }
}
