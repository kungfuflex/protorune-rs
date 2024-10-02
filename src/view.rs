use crate::constants;
use bitcoin::hashes::{ sha256d, Hash };
use bitcoin::{ OutPoint };
use metashrew::index_pointer::KeyValuePointer;
use std::sync::Arc;

#[derive(serde::Serialize)]
pub struct AddressOutpoints {
    pub outpoints: Vec<String>,
}

pub fn outpoints_by_address(address: Vec<u8>) -> String {
    let outpoints = constants::OUTPOINTS_FOR_ADDRESS.select(&address).get_list();
    let mut ret: AddressOutpoints = AddressOutpoints { outpoints: Vec::new() };
    for outpoint in outpoints {
        let _address = constants::OUTPOINT_SPENDABLE_BY.select(&outpoint).get();
        if address.len() == _address.len() {
            let final_outpoint: String = outpoint_from_bytes(&outpoint)
                .expect("Invalid outpoint")
                .to_string();
            ret.outpoints.push(final_outpoint);
        }
    }
    return serde_json::to_string_pretty(&ret).unwrap();
}

fn outpoint_from_bytes(bytes: &[u8]) -> Option<OutPoint> {
    // Ensure the byte array has the correct length (32 bytes for txid + 4 bytes for vout)
    if bytes.len() != 36 {
        return None; // Invalid length
    }

    // Split the bytes into txid (32 bytes) and vout (4 bytes)
    let (txid_bytes, vout_bytes) = bytes.split_at(32);

    // Convert the txid bytes into a sha256d::Hash (used by OutPoint)
    let txid = sha256d::Hash::from_slice(txid_bytes).ok()?;

    // Convert the vout bytes into a u32
    let vout = u32::from_le_bytes(vout_bytes.try_into().unwrap());

    // Create the OutPoint from txid and vout
    Some(OutPoint {
        txid: txid.into(),
        vout,
    })
}
