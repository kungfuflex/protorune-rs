use bitcoin::blockdata::block::Block;
use bitcoin::hashes::Hash;
use bitcoin::Network::{Bitcoin, Regtest};
use metashrew::index_pointer::{IndexPointer, KeyValuePointer};
use once_cell::sync::Lazy;
use std::sync::atomic::AtomicPtr;
use std::sync::Arc;

pub const RUNESTONE_TAG: u16 = 0x5d6a;
pub const OP_RETURN: u8 = 0x6a;
pub const GENESIS: u32 = 840000;
pub const NETWORK: bitcoin::Network = Bitcoin;

pub static HEIGHT_TO_BLOCKHASH: Lazy<IndexPointer> =
    Lazy::new(|| IndexPointer::from_keyword("/blockhash/byheight/"));
pub static BLOCKHASH_TO_HEIGHT: Lazy<IndexPointer> =
    Lazy::new(|| IndexPointer::from_keyword("/height/byblockhash/"));
pub static OUTPOINT_TO_RUNES: Lazy<IndexPointer> =
    Lazy::new(|| IndexPointer::from_keyword("/runes/byoutpoint/"));
pub static HEIGHT_TO_RUNES: Lazy<IndexPointer> =
    Lazy::new(|| IndexPointer::from_keyword("/runes/byheight/"));

pub static ADDRESS_TO_RUNES: Lazy<IndexPointer> =
    Lazy::new(|| IndexPointer::from_keyword("/runes/byaddress/"));

pub static ADDRESS_TO_PROTORUNES: Lazy<IndexPointer> =
    Lazy::new(|| IndexPointer::from_keyword("/protorunes/byaddress/"));

pub static HEIGHT_TO_PROTORUNES: Lazy<IndexPointer> =
    Lazy::new(|| IndexPointer::from_keyword("/protorunes/byheight/"));

pub static OUTPOINTS_FOR_ADDRESS: Lazy<IndexPointer> =
    Lazy::new(|| IndexPointer::from_keyword("/outpoint/byaddress/"));

pub static OUTPOINT_SPENDABLE_BY: Lazy<IndexPointer> =
    Lazy::new(|| IndexPointer::from_keyword("/outpoint/spendableby/"));
pub static OUTPOINT_TO_OUTPUT: Lazy<IndexPointer> =
    Lazy::new(|| IndexPointer::from_keyword("/output/byoutpoint/"));
pub static OUTPOINT_TO_HEIGHT: Lazy<IndexPointer> =
    Lazy::new(|| IndexPointer::from_keyword("/height/byoutpoint/"));
pub static HEIGHT_TO_TRANSACTION_IDS: Lazy<IndexPointer> =
    Lazy::new(|| IndexPointer::from_keyword("/txids/byheight"));

pub static SYMBOL: Lazy<IndexPointer> = Lazy::new(|| IndexPointer::from_keyword("/runes/symbol/"));
pub static CAP: Lazy<IndexPointer> = Lazy::new(|| IndexPointer::from_keyword("/runes/cap/"));
pub static SPACERS: Lazy<IndexPointer> = Lazy::new(|| IndexPointer::from_keyword("/runes/spaces/"));
pub static OFFSETEND: Lazy<IndexPointer> =
    Lazy::new(|| IndexPointer::from_keyword("/runes/offset/end/"));
pub static OFFSETSTART: Lazy<IndexPointer> =
    Lazy::new(|| IndexPointer::from_keyword("/runes/offset/start/"));
pub static HEIGHTSTART: Lazy<IndexPointer> =
    Lazy::new(|| IndexPointer::from_keyword("/runes/height/start/"));
pub static HEIGHTEND: Lazy<IndexPointer> =
    Lazy::new(|| IndexPointer::from_keyword("/runes/height/end/"));
pub static AMOUNT: Lazy<IndexPointer> = Lazy::new(|| IndexPointer::from_keyword("/runes/amount/"));
pub static MINTS_REMAINING: Lazy<IndexPointer> =
    Lazy::new(|| IndexPointer::from_keyword("/runes/mints-remaining/"));
pub static PREMINE: Lazy<IndexPointer> =
    Lazy::new(|| IndexPointer::from_keyword("/runes/premine/"));
pub static DIVISIBILITY: Lazy<IndexPointer> =
    Lazy::new(|| IndexPointer::from_keyword("/runes/divisibility/"));
pub static RUNE_ID_TO_HEIGHT: Lazy<IndexPointer> =
    Lazy::new(|| IndexPointer::from_keyword("/height/byruneid/"));
pub static ETCHINGS: Lazy<IndexPointer> = Lazy::new(|| IndexPointer::from_keyword("/runes/names"));
pub static RUNE_ID_TO_ETCHING: Lazy<IndexPointer> =
    Lazy::new(|| IndexPointer::from_keyword("/etching/byruneid/"));
pub static ETCHING_TO_RUNE_ID: Lazy<IndexPointer> =
    Lazy::new(|| IndexPointer::from_keyword("/runeid/byetching/"));

pub const SUBSIDY_HALVING_INTERVAL: u64 = 210_000;
pub const HEIGHT_INTERVAL: u64 = 17_500;

pub const MAX_BYTES_LEB128_INT: usize = 18;
