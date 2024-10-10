use std::io::Cursor;
use std::sync::Arc;

use crate::proto::protorune::{ OutpointResponse, WalletResponse, Output };
use crate::tables;
use bitcoin::consensus::Decodable;
use bitcoin::hashes::{ sha256d, Hash };
use bitcoin::OutPoint;
use metashrew::index_pointer::KeyValuePointer;
use protobuf::Message;

pub fn runes_by_address(address: Vec<u8>) -> WalletResponse {
    let outpoints = tables::OUTPOINTS_FOR_ADDRESS.select(&address).get_list();
    let mut ret: WalletResponse = WalletResponse::new();
    for outpoint in outpoints {
        let _address = tables::OUTPOINT_SPENDABLE_BY.select(&outpoint).get();
        if address.len() == _address.len() {
            let output: Output = Output::parse_from_bytes(
                &tables::OUTPOINT_TO_OUTPUT.select(&*&outpoint).get()
            ).unwrap();
            let mut cursor = Cursor::new(&*outpoint);
            let final_outpoint: OutPoint = OutPoint::consensus_decode(&mut cursor).unwrap();
            println!("{:?}", final_outpoint);
            // ret.outpoints.push(final_outpoint);
        }
    }
    return ret;
}
