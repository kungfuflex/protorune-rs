use crate::message::MessageContext;
use anyhow::Result;
use bitcoin::consensus::encode::serialize;
use bitcoin::{ block, Address, OutPoint, Script, ScriptBuf };
use bitcoin::blockdata::block::Block;
use bitcoin::hashes::Hash;
use metashrew_rs::{ flush, println, stdout };
use ordinals::Etching;
use ordinals::{ Artifact, Runestone };
use std::fmt::Write;
use std::sync::Arc;
use metashrew_rs::index_pointer::{KeyValuePointer};

pub mod balance_sheet;
pub mod constants;
pub mod message;
pub mod protoburn;
pub mod protostone;
#[cfg(test)]
pub mod tests;

pub struct Protorune(());

impl Protorune {
    pub fn index_etching(etching_optional: &Option<Etching>) {
        if let Some(etching) = etching_optional {
        }
    }

    pub fn index_runestone<T: MessageContext>(block: &Block) {
        for tx in block.txdata.iter() {
            if let Some(Artifact::Runestone(runestone)) = Runestone::decipher(tx) {
                Self::index_etching(&runestone.etching);
            }
        }
    }

    pub fn index_block<T: MessageContext>(block: Block, height: u32) -> Result<()> {
        constants::HEIGHT_TO_BLOCKHASH
            .select_value::<u32>(height)
            .set(Arc::new(block.block_hash().as_byte_array().to_vec()));
        constants::BLOCKHASH_TO_HEIGHT
            .select(&block.block_hash().as_byte_array().to_vec())
            .set_value::<u32>(height);
        for transaction in &block.txdata {
            let tx_id = transaction.txid();
            for (index, output) in transaction.output.iter().enumerate() {
                let outpoint = OutPoint {
                    txid: tx_id.clone(),
                    vout: index as u32,
                };
                let output_script: &ScriptBuf = &output.script_pubkey;
                if Address::from_script(&output_script, constants::NETWORK).is_ok() {
                    let outpoint_bytes: Vec<u8> = serialize(&outpoint);
                    let address = Address::from_script(&output_script, constants::NETWORK)?;
                    constants::OUTPOINTS_FOR_ADDRESS
                        .select(&address.to_string().into_bytes())
                        .append(Arc::new(outpoint_bytes.clone()));
                    constants::OUTPOINT_SPENDABLE_BY
                        .select(&outpoint_bytes.clone())
                        .set(Arc::new(address.to_string().into_bytes()));
                }
            }
        }
        Self::index_runestone::<T>(&block);
        let _protocol_tag = T::protocol_tag();
        println!("got block");
        flush();
        Ok(())
    }
}
