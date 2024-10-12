use bitcoin::Network::Bitcoin;

pub const RUNESTONE_TAG: u16 = 0x5d6a;
pub const OP_RETURN: u8 = 0x6a;
pub const GENESIS: u32 = 840000;
pub const NETWORK: bitcoin::Network = Bitcoin;
pub const MINIMUM_NAME: u128 = 99246114928149462;
pub const RESERVED_NAME: u128 = 6402364363415443603228541259936211926;
pub const TWENTY_SIX: u128 = 26;

pub const SUBSIDY_HALVING_INTERVAL: u64 = 210_000;
pub const HEIGHT_INTERVAL: u64 = 17_500;

pub const MAX_BYTES_LEB128_INT: usize = 18;
