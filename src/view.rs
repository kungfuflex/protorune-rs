use crate::{tables};
use bitcoin::hashes::{sha256d, Hash};
use bitcoin::OutPoint;
use metashrew::index_pointer::KeyValuePointer;

#[derive(serde::Serialize, Debug)]
pub struct AddressOutpoints {
    pub outpoints: Vec<String>,
}

pub struct View(());

impl View {
    pub fn outpoints_by_address(address: Vec<u8>) -> String {
        let outpoints = tables::OUTPOINTS_FOR_ADDRESS.select(&address).get_list();
        let mut ret: AddressOutpoints = AddressOutpoints {
            outpoints: Vec::new(),
        };
        for outpoint in outpoints {
            let _address = tables::OUTPOINT_SPENDABLE_BY.select(&outpoint).get();
            if address.len() == _address.len() {
                let final_outpoint: String = Self::outpoint_from_bytes(&outpoint)
                    .expect("Invalid outpoint")
                    .to_string();
                ret.outpoints.push(final_outpoint);
            }
        }
        return serde_json::to_string_pretty(&ret).unwrap();
    }

    fn outpoint_from_bytes(bytes: &[u8]) -> Option<OutPoint> {
        if bytes.len() != 36 {
            return None;
        }

        let (txid_bytes, vout_bytes) = bytes.split_at(32);

        let txid = sha256d::Hash::from_slice(txid_bytes).ok()?;

        let vout = u32::from_le_bytes(vout_bytes.try_into().unwrap());

        Some(OutPoint {
            txid: txid.into(),
            vout,
        })
    }
}
