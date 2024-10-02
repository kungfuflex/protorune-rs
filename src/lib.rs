use crate::message::MessageContext;
use anyhow::{anyhow, Result};
use bitcoin::blockdata::block::Block;
use bitcoin::consensus::encode::serialize;
use bitcoin::hashes::Hash;
use bitcoin::{block, Address, OutPoint, Script, ScriptBuf, Transaction};
use metashrew::index_pointer::KeyValuePointer;
use metashrew::{flush, println, stdout};
use ordinals::Etching;
use ordinals::{Artifact, Runestone};
use protostone::{add_to_indexable_protocols, initialized_protocol_index};
use std::fmt::Write;
use std::sync::Arc;

pub mod balance_sheet;
pub mod byte_utils;
pub mod constants;
pub mod message;
pub mod protoburn;
pub mod protostone;
#[cfg(test)]
pub mod tests;
pub mod view;

pub struct Protorune(());

impl Protorune {
    pub fn index_runestone<T: MessageContext>(
        tx: &Transaction,
        runestone: &Runestone,
        height: u32,
    ) -> Result<()> {
        if let Some(etching) = runestone.etching.as_ref() {
            Self::index_etching(etching)?;
        }
        Ok(())
    }
    pub fn index_etching(etching: &Etching) -> Result<()> {
        Ok(())
    }

    pub fn index_unspendables<T: MessageContext>(block: &Block, height: u32) -> Result<()> {
        for tx in block.txdata.iter() {
            if let Some(Artifact::Runestone(ref runestone)) = Runestone::decipher(tx) {
                Self::index_runestone::<T>(tx, runestone, height)?
            }
        }
        Ok(())
    }
    pub fn index_spendables(txdata: &Vec<Transaction>) -> Result<()> {
        for transaction in txdata {
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
        Ok(())
    }

    pub fn index_transaction_ids(block: &Block, height: u32) -> Result<()> {
        let ptr = constants::HEIGHT_TO_TRANSACTION_IDS.select_value::<u32>(height);
        for tx in &block.txdata {
            ptr.append(Arc::new(tx.txid().as_byte_array().to_vec()));
        }
        Ok(())
    }
    pub fn index_outpoints(block: &Block, height: u32) -> Result<()> {
        for tx in &block.txdata {
            let ptr = constants::OUTPOINT_TO_HEIGHT.select(&tx.txid().as_byte_array().to_vec());
            for i in 0..tx.output.len() {
                ptr.select_value(i as u32).set_value(height);
            }
        }
        Ok(())
    }
    pub fn index_block<T: MessageContext>(block: Block, height: u32) -> Result<()> {
        initialized_protocol_index().map_err(|e| anyhow!(e.to_string()))?;
        add_to_indexable_protocols(T::protocol_tag()).map_err(|e| anyhow!(e.to_string()))?;
        constants::HEIGHT_TO_BLOCKHASH
            .select_value::<u32>(height)
            .set(Arc::new(block.block_hash().as_byte_array().to_vec()));
        constants::BLOCKHASH_TO_HEIGHT
            .select(&block.block_hash().as_byte_array().to_vec())
            .set_value::<u32>(height);
        Self::index_spendables(&block.txdata)?;
        Self::index_transaction_ids(&block, height)?;
        Self::index_outpoints(&block, height)?;
        Self::index_unspendables::<T>(&block, height)?;
        println!("got block");
        flush();
        Ok(())
    }
}
