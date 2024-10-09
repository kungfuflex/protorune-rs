use std::sync::Arc;

use crate::proto::protorune::{ WalletResponse };
use crate::tables;
use bitcoin::hashes::{ sha256d, Hash };
use bitcoin::OutPoint;
use metashrew::index_pointer::KeyValuePointer;

pub fn runes_by_address(address: Vec<u8>) -> WalletResponse {
    let outpoints = tables::OUTPOINTS_FOR_ADDRESS.select(&address).get_list();
    let mut ret: WalletResponse = WalletResponse::new();
    for outpoint in outpoints {
        let _address = tables::OUTPOINT_SPENDABLE_BY.select(&outpoint).get();
        if address.len() == _address.len() {
            let final_outpoint: OutPoint = OutPoint::consensus_decode(&outpoint).expect(
                "Invalid outpoint"
            );
            ret.outpoints.push(final_outpoint);
        }
    }
    return ret;
}
