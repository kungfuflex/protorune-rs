use protorune_support::{
    balance_sheet::{BalanceSheet, ProtoruneRuneId},
    byte_utils::ByteUtils,
    rune_transfer::{OutgoingRunes, RuneTransfer}
};
use crate::{
    message::{MessageContext, MessageContextParcel},
    protoburn::{Protoburn, Protoburns},
};
use anyhow::{anyhow, Result};
use bitcoin::{Block, Transaction, Txid};
use metashrew::index_pointer::{AtomicPointer, IndexPointer};
use ordinals::{runestone::tag::Tag, varint, Edict, RuneId, Runestone};
use std::collections::{HashMap, HashSet};

static mut PROTOCOLS: Option<HashSet<u128>> = None;

pub fn initialized_protocol_index() -> Result<()> {
    unsafe { PROTOCOLS = Some(HashSet::new()) }
    Ok(())
}

pub fn next_protostone_edict_id(
    id: &ProtoruneRuneId,
    block: u128,
    tx: u128,
) -> Option<ProtoruneRuneId> {
    Some(ProtoruneRuneId {
        block: id.block.checked_add(block)?,
        tx: if block == 0 {
            id.tx.checked_add(tx)?
        } else {
            tx
        },
    })
}

#[derive(Clone, Default, PartialEq, Debug)]
pub struct ProtostoneEdict {
    pub id: ProtoruneRuneId,
    pub amount: u128,
    pub output: u128,
}

impl From<ProtostoneEdict> for Edict {
    fn from(v: ProtostoneEdict) -> Edict {
        Edict {
            id: RuneId {
                block: v.id.block as u64,
                tx: v.id.tx as u32,
            },
            amount: v.amount,
            output: v.output as u32,
        }
    }
}

pub fn into_protostone_edicts(v: Vec<Edict>) -> Vec<ProtostoneEdict> {
    v.into_iter().map(|v| v.into()).collect()
}

pub fn make_edict_set_size_error() -> anyhow::Error {
    anyhow!("edict values did not appear in sets of four")
}

pub fn protostone_edicts_from_integers(v: &Vec<u128>) -> Result<Vec<ProtostoneEdict>> {
    let mut last = ProtoruneRuneId::default();
    let mut result: Vec<ProtostoneEdict> = vec![];
    for chunk in v.chunks(4) {
        match chunk {
            [block, tx, amount, output] => {
                let edict = ProtostoneEdict {
                    id: next_protostone_edict_id(&last, *block, *tx)
                        .ok_or("")
                        .map_err(|_| anyhow!("edict processing failed -- overflow"))?,
                    amount: *amount,
                    output: *output,
                };
                last = edict.id.clone();
                result.push(edict);
            }
            _ => {
                return Err(make_edict_set_size_error());
            }
        }
    }
    Ok(result)
}

pub fn add_to_indexable_protocols(protocol_tag: u128) -> Result<()> {
    unsafe {
        if let Some(set) = PROTOCOLS.as_mut() {
            set.insert(protocol_tag);
        }
    }
    Ok(())
}

/*
fn has_protocol(protocol_tag: &u128) -> Result<bool> {
    unsafe {
        if let Some(set) = PROTOCOLS.as_mut() {
            let contains = set.contains(protocol_tag);
            return Ok(contains);
        }
    }
    Ok(false)
}
*/

fn next_two<T, I>(iter: &mut I) -> Option<(T, T)>
where
    I: Iterator<Item = T>,
{
    let first = iter.next()?;
    let second = iter.next()?;
    Some((first, second))
}

fn take_n<T, I: Iterator<Item = T>>(iter: &mut I, n: usize) -> Option<Vec<T>> {
    let mut i = 0;
    let mut result: Vec<T> = Vec::<T>::new();
    loop {
        if i == n {
            break;
        }
        if let Some(v) = iter.next() {
            result.push(v);
            i += 1;
        } else {
            break;
        }
    }
    if i == n {
        Some(result)
    } else {
        None
    }
}

pub fn to_fields(values: &Vec<u128>) -> HashMap<u128, Vec<u128>> {
    let mut map: HashMap<u128, Vec<u128>> = HashMap::new();
    let mut iter = values
        .into_iter()
        .map(|v| *v)
        .collect::<Vec<u128>>()
        .into_iter();
    while let Some((key, value)) = next_two(&mut iter) {
        if key == 0u128 {
            let remaining_values: Vec<u128> = iter.collect::<Vec<u128>>();
            map.entry(key).or_insert_with(Vec::new).push(value);
            map.get_mut(&key).unwrap().extend(remaining_values);
            break;
        } else {
            map.entry(key).or_insert_with(Vec::new).push(value);
        }
    }
    map
}

#[derive(Clone, PartialEq, Debug)]
pub struct Protostone {
    pub burn: Option<u128>,
    pub message: Vec<u8>,
    pub edicts: Vec<ProtostoneEdict>,
    pub refund: Option<u32>,
    pub pointer: Option<u32>,
    pub from: Option<u32>,
    pub protocol_tag: u128,
}

/*
fn varint_byte_len(input: &Vec<u8>, n: u128) -> Result<usize> {
    let mut cloned = input.clone();
    for _i in 0..n {
        let (_, size) =
            varint::decode(&cloned.as_slice()).map_err(|_| anyhow!("varint decode error"))?;
        cloned.drain(0..size);
    }

    Ok(input.len() - cloned.len())
}
*/

/// This takes in an arbituary amount of bytes, and
/// converts it in a list of u128s, making sure we don't
/// write to the 16th byte of the u128.
///
/// To ensure the range of bytearrays does not exclude
/// any bitfields within its terminal bytes, we choose a maximum length f
/// or a u128 value within a u128[] intended for interpretation as a u8[] to 15 bytes.
/// This allows us to safely model an arbitrary bytearray within the Runestone paradigm.
pub fn split_bytes(v: &Vec<u8>) -> Vec<u128> {
    let mut result: Vec<Vec<u8>> = vec![];
    v.iter().enumerate().for_each(|(i, b)| {
        if i % 15 == 0 {
            result.push(Vec::<u8>::new());
        }
        result.last_mut().unwrap().push(*b);
    });
    result
        .iter_mut()
        .map(|v| {
            v.resize(std::mem::size_of::<u128>(), 0u8);
            return u128::from_le_bytes((&v[0..16]).try_into().unwrap());
        })
        .collect::<Vec<u128>>()
}

pub fn join_to_bytes(v: &Vec<u128>) -> Vec<u8> {
    let mut result: Vec<u8> = vec![];
    for (_, integer) in v.iter().enumerate() {
        // if i != v.len() - 1 {
        result.extend(<u128 as ByteUtils>::snap_to_15_bytes(*integer))
        // we don't insert a 0 byte for the 16th byte
        // } else {
        //     result.extend(<u128 as ByteUtils>::to_aligned_bytes(*integer))
        // }
    }
    result
}

impl Protostone {
    pub fn append_edicts(&mut self, edicts: Vec<Edict>) {
        self.edicts = into_protostone_edicts(edicts);
    }
    pub fn is_message(&self) -> bool {
        !self.message.is_empty()
    }
    /// Enciphers a protostone into a vector of u128s
    /// NOTE: This is not LEB encoded
    pub fn to_integers(&self) -> Result<Vec<u128>> {
        let mut payload = Vec::<u128>::new();

        if let Some(burn) = self.burn {
            payload.push(Tag::Burn.into());
            payload.push(burn.into());
        }
        if let Some(pointer) = self.pointer {
            payload.push(Tag::ProtoPointer.into());
            payload.push(pointer.into());
        }
        if let Some(refund) = self.refund {
            payload.push(Tag::Refund.into());
            payload.push(refund.into());
        }
        if let Some(from) = self.from.as_ref() {
            payload.push(Tag::From.into());
            payload.push((*from).into());
        }
        if !self.message.is_empty() {
            for item in split_bytes(&self.message) {
                payload.push(Tag::Message.into());
                payload.push(item);
            }
        }
        if !self.edicts.is_empty() {
            payload.push(Tag::Body.into());
            let mut edicts = self.edicts.clone();
            edicts.sort_by_key(|edict| edict.id);

            let mut previous = ProtoruneRuneId::default();
            for edict in edicts {
                let (block, tx) = previous
                    .delta(edict.id.into())
                    .ok_or("")
                    .map_err(|_| anyhow!("invalid delta"))?;
                payload.push(block);
                payload.push(tx);
                payload.push(edict.amount);
                payload.push(edict.output.into());
                previous = edict.id.into();
            }
        }
        Ok(payload)
    }
    pub fn process_message<T: MessageContext>(
        &self,
        atomic: &mut AtomicPointer,
        transaction: &Transaction,
        txindex: u32,
        block: &Block,
        height: u64,
        _runestone_output_index: u32,
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
                runes: RuneTransfer::from_balance_sheet(initial_sheet.clone()),
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
                runtime_balances: Box::new(
                    balances_by_output
                        .get(&u32::MAX)
                        .map(|v| v.clone())
                        .unwrap_or_else(|| BalanceSheet::default()),
                ),
                sheets: Box::new(BalanceSheet::default()),
            };
            let pointer = self.pointer.unwrap_or_else(|| default_output);
            let refund_pointer = self.refund.unwrap_or_else(|| default_output);
            match T::handle(&parcel) {
                Ok(values) => match values.reconcile(balances_by_output, vout, pointer) {
                    Ok(_) => atomic.commit(),
                    Err(_) => {
                        let sheet = balances_by_output
                            .get(&vout)
                            .map(|v| v.clone())
                            .unwrap_or_else(|| BalanceSheet::default());
                        balances_by_output.remove(&vout);
                        if !balances_by_output.contains_key(&refund_pointer) {
                            balances_by_output.insert(refund_pointer, BalanceSheet::default());
                        }
                        sheet.pipe(balances_by_output.get_mut(&refund_pointer).unwrap());
                        atomic.rollback()
                    }
                },
                Err(_) => {
                    atomic.rollback();
                }
            }
        }
        Ok(())
    }
    pub fn from_fields_and_tag(map: &HashMap<u128, Vec<u128>>, protocol_tag: u128) -> Result<Self> {
        Ok(Protostone {
            burn: map.get(&Tag::Burn.into()).map(|v| v[0] as u128),
            message: join_to_bytes(
                &map.get(&Tag::Message.into())
                    .map(|v| v.clone())
                    .unwrap_or_else(|| Vec::<u128>::new()),
            ),
            refund: map.get(&Tag::Refund.into()).map(|v| v[0] as u32),
            pointer: map.get(&Tag::ProtoPointer.into()).map(|v| v[0] as u32),
            protocol_tag,
            from: map.get(&Tag::From.into()).map(|v| v[0] as u32),
            edicts: map
                .get(&0u128)
                .map(|list| -> Result<Vec<ProtostoneEdict>> {
                    protostone_edicts_from_integers(&list)
                })
                .and_then(|v| v.ok())
                .unwrap_or_else(|| vec![]),
        })
    }

    pub fn from_runestone(runestone: &Runestone) -> Result<Vec<Self>> {
        if let None = runestone.protocol.as_ref() {
            return Ok(vec![]);
        }
        let protostone_raw = runestone
            .protocol
            .clone()
            .ok_or(anyhow!("no protostone field in runestone"))?;

        Ok(Protostone::decipher(&protostone_raw)?)
    }

    /// Gets a vector of Protostones from an arbituary vector of bytes
    ///
    /// protostone_raw: LEB encoded Protostone
    /// num_outputs: needed to check that the edicts of the protostone do not exceed the
    pub fn decipher(values: &Vec<u128>) -> Result<Vec<Protostone>> {
        let raw: Vec<u8> = join_to_bytes(values);
        let mut iter = Runestone::integers(&raw)?.into_iter();
        let mut result: Vec<Protostone> = vec![];
        loop {
            if let Some(protocol_tag) = iter.next() {
                // if protocol_tag == 0 then break, since we don't allow protocol tag to equal zero anyways.
                // also this means we have postfix zeroes in the last u128
                if protocol_tag == 0 {
                    break;
                }
                if let Some(length) = iter.next() {
                    result.push(Protostone::from_fields_and_tag(
                        &to_fields(
                            &(take_n(&mut iter, length.try_into()?)
                                .ok_or("")
                                .map_err(|_| anyhow!("less values than expected")))?,
                        ),
                        protocol_tag,
                    )?);
                } else {
                    break;
                }
            } else {
                break;
            }
        }
        Ok(result)
    }

    // when encoding a Protostone into the first layer of LEB encoding, we need to make sure it only uses the first
}

pub trait Protostones {
    fn burns(&self) -> Result<Vec<Protoburn>>;
    fn process_burns(
        &self,
        atomic: &mut AtomicPointer,
        runestone: &Runestone,
        runestone_output_index: u32,
        balances_by_output: &HashMap<u32, BalanceSheet>,
        proto_balances_by_output: &mut HashMap<u32, BalanceSheet>,
        default_output: u32,
        txid: Txid,
    ) -> Result<()>;
    fn encipher(&self) -> Result<Vec<u128>>;
}

/// returns the values in a LEB encoded stream
pub fn encode_varint_list(values: &Vec<u128>) -> Vec<u8> {
    let mut result = Vec::<u8>::new();
    for value in values {
        varint::encode_to_vec(*value, &mut result);
    }
    result
}

impl Protostones for Vec<Protostone> {
    fn encipher(&self) -> Result<Vec<u128>> {
        let mut values = Vec::<u128>::new();
        for stone in self {
            values.push(stone.protocol_tag);
            let varints = stone.to_integers()?;
            values.push(varints.len() as u128);
            values.extend(&varints);
        }
        Ok(split_bytes(&encode_varint_list(&values)))
    }
    fn burns(&self) -> Result<Vec<Protoburn>> {
        Ok(self
            .into_iter()
            .filter(|stone| stone.burn.is_some())
            .map(|stone| Protoburn {
                tag: stone.burn.map(|v| v as u128),
                pointer: stone.pointer,
                from: stone.from.map(|v| vec![v]),
            })
            .collect())
    }
    fn process_burns(
        &self,
        atomic: &mut AtomicPointer,
        runestone: &Runestone,
        runestone_output_index: u32,
        balances_by_output: &HashMap<u32, BalanceSheet>,
        proto_balances_by_output: &mut HashMap<u32, BalanceSheet>,
        default_output: u32,
        txid: Txid,
    ) -> Result<()> {
        let mut burns = self.burns()?;
        burns.process(
            atomic,
            runestone,
            runestone_output_index,
            balances_by_output,
            proto_balances_by_output,
            default_output,
            txid,
        )?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    /// Lets say we have a protostone defined as follows: vec<u128>![1 4 83 0 91 3]. This is a protostone with a protocol tag of 1, a length of 4, tag 83 (burn) is 0, tag 91 (pointer) is 3.
    /// Encoding:
    /// 1. Protocol step: Each u128 is LEB encoded. Each u128 becomes a vector of up to 16 bytes and is then concatenated together. LEB saves space by allowing smaller numbers to be one byte.
    ///         type: vec<u8>
    ///         [1 4 83 0 91 3]
    /// 2. Compression step: Combine the vec<u8> into a vec<u128> where we don't use the 16th byte. We should make the endianess such that the runes encodes is most efficient
    ///         type: vec<u128>. In this case, we can fit all our numbers into one u128.
    ///         this protostone becomes one u128 with bytes [1 4 83 0 91 3 0 0 0 0 0 0 0 0 0 0] or [0 0 0 0 0 0 0 0 0 0 3 91 0 83 4 1]
    ///         machine is little endian (wasm is little endian) = then we want to store it [1 4 83 0 91 3 0 0 0 0 0 0 0 0 0 0]
    ///         if machine was big endian = then we want to store it [0 0 0 0 0 0 0 0 0 0 3 91 0 83 4 1]
    ///
    ///         CONCLUSION:
    ///         since we are building to wasm, and wasm is little endian, we should store it with the data bytes at the lower memory address, so [1 4 83 0 91 3 0 0 0 0 0 0 0 0 0 0]
    /// 3. (Runes) LEB Encode each u128. The smaller the u128 the better.

    /// Assume runes already read the proto from tags.
    /// Decoding: proto is a vec<u128> (arbituary vector of u128 that we have to decode into a protostone) vec![u128([1 4 83 0 91 3 0 0 0 0 0 0 0 0 0 0])]
    /// 1. Undo the compression: convert each u128 into a vec<u8> and then concat to one array.
    ///         Important notes:
    ///          - We need to strip the 16th byte from each u128 to follow the spec
    ///          - [REMOVED] For the very last u128, we strip all postfix zeroes -- we don't want to do this because what if our input was like this?: vec![u128([1 4 91 3 83 0 0 0 0 0 0 0 0 0 0 0])]
    ///         input: vec![u128([1 4 83 0 91 3 0 0 0 0 0 0 0 0 0 0])]
    ///         output: vec<u8>![1 4 83 0 91 3 0 0 0 0 0 0 0 0 0]
    ///
    /// 2. Now we can LEB decode this vector of bytes into a vector of u128s. Note in this example, all numbers are less than 7 bits so their LEB representation is the same as the original u128.
    ///         input: vec<u8>![1 4 83 0 91 3 0 0 0 0 0 0 0 0 0]
    ///         output: vec<u128>![1 4 83 0 91 3 0 0 0 0 0 0 0 0 0]
    ///
    use super::*;

    #[test]
    fn test_protostone_encipher_burn() {
        let protostones = vec![Protostone {
            burn: Some(1u128),
            edicts: vec![],
            pointer: Some(3),
            refund: None,
            from: None,
            protocol_tag: 13, // must be 13 when protoburn
            message: vec![],
        }];

        let protostone_enciphered = protostones.encipher().unwrap();

        let protostone_decipered = Protostone::decipher(&protostone_enciphered).unwrap();

        assert_eq!(protostones, protostone_decipered);
    }

    #[test]
    fn test_protostone_encipher_edict() {
        let protostones = vec![Protostone {
            burn: Some(0u128),
            edicts: vec![ProtostoneEdict {
                id: ProtoruneRuneId {
                    block: 8400000,
                    tx: 1,
                },
                amount: 123456789,
                output: 2,
            }],
            pointer: Some(3),
            refund: None,
            from: None,
            protocol_tag: 1,
            message: vec![],
        }];

        let protostone_enciphered = protostones.encipher().unwrap();

        let protostone_decipered = Protostone::decipher(&protostone_enciphered).unwrap();

        assert_eq!(protostones, protostone_decipered);
    }

    #[test]
    fn test_protostone_encipher_multiple_u128() {
        let protostones = vec![Protostone {
            burn: None,
            edicts: vec![],
            pointer: Some(3),
            refund: None,
            from: None,
            protocol_tag: 1,
            message: vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 0, 0, 0, 0, 0, 0], // what we pass in should be well defined by the subprotocol
        }];

        let protostone_enciphered = protostones.encipher().unwrap();

        let protostone_decipered = Protostone::decipher(&protostone_enciphered).unwrap();

        assert_eq!(protostones, protostone_decipered);
    }

    #[test]
    fn test_protostone_encipher_multiple_protostones() {
        let protostones = vec![
            Protostone {
                burn: Some(1u128),
                edicts: vec![],
                pointer: Some(3),
                refund: None,
                from: None,
                protocol_tag: 13,
                message: vec![],
            },
            Protostone {
                burn: Some(1u128),
                edicts: vec![],
                pointer: Some(2),
                refund: None,
                from: None,
                protocol_tag: 3,
                message: vec![100, 11, 112, 113, 114, 115, 116, 117, 118, 0, 0, 0, 0, 0, 0],
            },
        ];

        let protostone_enciphered = protostones.encipher().unwrap();

        let protostone_decipered = Protostone::decipher(&protostone_enciphered).unwrap();

        assert_eq!(protostones, protostone_decipered);
    }
}
