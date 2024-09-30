use crate::message::MessageContext;
use anyhow::Result;
use bitcoin::blockdata::block::Block;
use bitcoin::hashes::Hash;
use metashrew_rs::{flush, println, stdout};
use ordinals::Etching;
use ordinals::{Artifact, Runestone};
use std::fmt::Write;
use std::sync::Arc;

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
        if let Some(etching) = etching_optional {}
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

        if height >= constants::GENESIS {
            Self::index_runestone::<T>(&block);
        }

        let _protocol_tag = T::protocol_tag();
        println!("got block");
        flush();
        Ok(())
    }
}
