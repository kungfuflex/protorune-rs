use crate::message::MessageContext;
use anyhow::{anyhow, Result};
use bitcoin::blockdata::block::Block;
use bitcoin::consensus::Decodable;
use bitcoin::hashes::Hash;
use metashrew_rs::{flush, index_pointer::IndexPointer, initialize, input, println, stdout};
use ordinals::{Artifact, Runestone};
use std::fmt::Write;
use std::sync::Arc;

pub mod message;

pub struct Protorune(());

impl Protorune {
    pub fn index_block<T: MessageContext>() -> Result<()> {
        initialize();
        let data = input();
        let height = u32::from_le_bytes(
            (&data[0..4])
                .try_into()
                .map_err(|_| anyhow!("not a metashrew input payload"))?,
        );
        let mut reader = &data[4..];
        let block =
            Block::consensus_decode(&mut reader).map_err(|_| anyhow!("failed to parse block"))?;
        IndexPointer::from_keyword("/blockhash/byheight/")
            .select_value::<u32>(height)
            .set(Arc::new(block.block_hash().as_byte_array().to_vec()));
        let _runestones: Vec<Option<Artifact>> = block
            .txdata
            .iter()
            .map(|tx| Runestone::decipher(tx))
            .collect();
        let _protocol_tag = T::protocol_tag();
        println!("got block");
        flush();
        Ok(())
    }
}
