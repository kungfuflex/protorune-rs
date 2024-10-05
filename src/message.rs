use std::u128;
use metashrew::index_pointer::AtomicPointer;
use crate::balance_sheet::{ProtoruneRuneId, BalanceSheet};
use std::collections::{HashMap};
use crate::tables::{RuneTable};
use bitcoin::{ Txid, Transaction, Block, OutPoint };

pub trait MessageContext {
    fn handle(parcel: Box<MessageContextParcel>) -> bool;
    fn protocol_tag() -> u128;
}

#[derive(Clone, Default)]
pub struct IncomingRune {
  pub rune: ProtoruneRuneId,
  pub amount: u128
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
  pub runtime_balances: Box<BalanceSheet>
}

