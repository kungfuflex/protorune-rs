use bitcoin::blockdata::block::Block;
use bitcoin::consensus::Decodable;
use bitcoin::hashes::Hash;
use std::fmt::Write;
use metashrew_rs::{
  index_pointer::{IndexPointer},
  initialize,
  input,
  println,
  stdout,
  flush
};
use std::sync::Arc;
use crate::message::MessageContext;
use ordinals::{Runestone};
use anyhow::{anyhow, Result};

pub mod message;

pub struct Protorune(());

impl Protorune {
  pub fn index_block<T: MessageContext>() -> Result<()> {
    initialize();
    let data = input();
    let height = u32::from_le_bytes((&data[0..4]).try_into().map_err(|_| anyhow!("not a metashrew input payload"))?);
    let mut reader = &data[4..];
    let block = Block::consensus_decode(&mut reader).map_err(|_| anyhow!("failed to parse block"))?;
    IndexPointer::from_keyword("/blockhash/byheight/").select_value::<u32>(height).set(Arc::new(block.block_hash().as_byte_array().to_vec()));
    let _protocol_tag = T::protocol_tag();
    println!("got block");
    flush();
    Ok(())
  }
}
