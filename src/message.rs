use crate::balance_sheet::{BalanceSheet, ProtoruneRuneId};
use crate::tables::RuneTable;
use bitcoin::{Block, OutPoint, Transaction, Txid};
use metashrew::index_pointer::AtomicPointer;
use std::collections::HashMap;
use std::u128;

pub trait MessageContext {
    fn handle(parcel: Box<MessageContextParcel>) -> bool;
    fn protocol_tag() -> u128;
}

#[derive(Clone, Default)]
pub struct IncomingRune {
    pub rune: ProtoruneRuneId,
    pub amount: u128,
}

#[derive(Clone)]
pub struct MessageContextParcel {
    pub atomic: AtomicPointer,
    pub runes: Vec<IncomingRune>,
    pub transaction: Transaction,
    pub block: Block,
    pub height: u64,
    pub outpoint: OutPoint,
    pub pointer: OutPoint,
    pub refund_pointer: OutPoint,
    pub calldata: Vec<u8>,
    pub txid: Txid,
    pub base_sheet: Box<BalanceSheet>,
    pub table: Box<RuneTable>,
    pub sheets: Box<HashMap<u32, BalanceSheet>>,
    pub txindex: u32,
    pub runtime_balances: Box<BalanceSheet>,
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
            pointer: OutPoint::null(),
            refund_pointer: OutPoint::null(),
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
