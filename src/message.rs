use crate::balance_sheet::{BalanceSheet, ProtoruneRuneId};
use crate::incoming_rune::IncomingRune;
use crate::tables::RuneTable;
use crate::utils::consensus_encode;
use anyhow::Result;
use bitcoin::{Block, OutPoint, Transaction, Txid};
use metashrew::index_pointer::AtomicPointer;
use std::collections::HashMap;
use std::u128;

pub trait MessageContext {
    fn handle(parcel: Box<MessageContextParcel>) -> bool;
    fn protocol_tag() -> u128;
}

#[derive(Clone)]
pub struct MessageContextParcel {
    pub atomic: AtomicPointer,
    pub runes: Vec<IncomingRune>,
    pub transaction: Transaction,
    pub block: Block,
    pub height: u64,
    pub outpoint: OutPoint,
    pub pointer: u32,
    pub refund_pointer: u32,
    pub calldata: Vec<u8>,
    pub txid: Txid,
    pub base_sheet: Box<BalanceSheet>,
    pub table: Box<RuneTable>,
    pub sheets: Box<HashMap<u32, BalanceSheet>>,
    pub txindex: u32,
    pub runtime_balances: Box<BalanceSheet>,
}

pub trait ToBytes {
    fn try_to_bytes(&self) -> Result<Vec<u8>>;
}

impl ToBytes for OutPoint {
    fn try_to_bytes(&self) -> Result<Vec<u8>> {
        Ok(consensus_encode(self)?)
    }
}

impl Default for MessageContextParcel {
    fn default() -> MessageContextParcel {
        let block = bitcoin::constants::genesis_block(bitcoin::Network::Bitcoin);
        MessageContextParcel {
            atomic: AtomicPointer::default(),
            runes: Vec::<IncomingRune>::default(),
            transaction: block.txdata[0].clone(),
            block: block.clone(),
            height: 0,
            outpoint: OutPoint::null(),
            pointer: 0,
            refund_pointer: 0,
            calldata: Vec::<u8>::default(),
            txid: block.txdata[0].txid(),
            base_sheet: Box::new(BalanceSheet::default()),
            table: Box::new(RuneTable::default()),
            sheets: Box::new(HashMap::<u32, BalanceSheet>::default()),
            txindex: 0,
            runtime_balances: Box::new(BalanceSheet::default()),
        }
    }
}
