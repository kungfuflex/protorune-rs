use crate::message::MessageContext;
use anyhow::{anyhow, Result};
use bitcoin::blockdata::block::Block;
use bitcoin::consensus::Decodable;
use bitcoin::hashes::Hash;
use metashrew_rs::{flush, index_pointer::IndexPointer, initialize, input, println, stdout};
use ordinals::{Artifact, Runestone};
use std::fmt::Write;
use std::sync::Arc;
use wasm_bindgen_test::*;
mod constants;

pub mod message;
pub mod protoburn;
pub mod protostone;
#[cfg(test)]
pub mod tests;

pub struct Protorune(());

impl Protorune {
    pub fn index_runestone<T: MessageContext>(block: Block) {
        let mut runestones: Vec<Artifact> = Vec::new();
        for tx in block.txdata.iter() {
            if let Some(runestone) = Runestone::decipher(tx) {
                runestones.push(runestone);
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

        if (height >= constants::GENESIS) {
            Self::index_runestone::<T>(block);
        }

        let _protocol_tag = T::protocol_tag();
        println!("got block");
        flush();
        Ok(())
    }
}

#[cfg(test)]
pub fn hello_world() -> String {
    String::from("hello world")
}
