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

pub mod message;
pub mod protoburn;
pub mod protostone;
#[cfg(test)]
pub mod tests;

pub struct Protorune(());

impl Protorune {
    pub fn index_block<T: MessageContext>(block: Block, height: u32) -> Result<()> {
        IndexPointer::from_keyword("/blockhash/byheight/")
            .select_value::<u32>(height)
            .set(Arc::new(block.block_hash().as_byte_array().to_vec()));
        flush();
        let _runestones: Vec<Option<Artifact>> = block
            .txdata
            .iter()
            .map(|tx| Runestone::decipher(tx))
            .collect();
        let _protocol_tag = T::protocol_tag();
        println!("got block");
        Ok(())
    }
}

#[cfg(test)]
pub fn hello_world() -> String {
    String::from("hello world")
}
