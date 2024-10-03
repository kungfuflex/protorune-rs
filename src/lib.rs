use crate::message::MessageContext;
use anyhow::{anyhow, Ok, Result};
use bitcoin::blockdata::block::Block;
use bitcoin::hashes::Hash;
use bitcoin::{TxOut, Address, OutPoint, ScriptBuf, Transaction};
use metashrew::index_pointer::KeyValuePointer;
use metashrew::{flush, println, stdout};
use ordinals::{Artifact, Runestone};
use ordinals::{Edict, Etching, RuneId};
use protostone::{add_to_indexable_protocols, initialized_protocol_index, Protostone, Protostones};
use std::fmt::Write;
use std::sync::Arc;
use crate::balance_sheet::{BalanceSheet};
use std::collections::{HashMap};
use crate::utils::consensus_encode;

pub mod balance_sheet;
pub mod byte_utils;
pub mod constants;
pub mod message;
pub mod protoburn;
pub mod protostone;
pub mod tables;
pub mod utils;
#[cfg(test)]
pub mod tests;
pub mod view;

pub struct Protorune(());

impl Protorune {
    pub fn index_runestone<T: MessageContext>(
        tx: &Transaction,
        runestone: &Runestone,
        height: u64,
        index: u32,
    ) -> Result<()> {
        let sheets: Vec<BalanceSheet> = tx.input.iter().map(|input| Ok(BalanceSheet::load(&tables::OUTPOINT_TO_RUNES.select(&consensus_encode(&input.previous_output)?)))).collect::<Result<Vec<BalanceSheet>>>()?;
        let balance_sheet = BalanceSheet::concat(sheets);
        let mut balances_by_output = HashMap::<u32, BalanceSheet>::new();
        if let Some(etching) = runestone.etching.as_ref() {
            Self::index_etching(etching, index, height)?;
        }
        Self::process_edicts(&runestone.edicts, &mut balances_by_output, &balance_sheet, &tx.output)?;
        Self::handle_leftover_runes(&balance_sheet, &mut balances_by_output)?;
        balances_by_output.into_iter().fold(Ok(()), |r, (vout, sheet)| -> Result<()> {
          r?;
          let outpoint = OutPoint::new(tx.txid(), vout);
          sheet.save(&tables::OUTPOINT_TO_RUNES.select(&consensus_encode(&outpoint)?), false);
          Ok(())
        })?;
        Ok(())
    }
    pub fn process_edict(edict: &Edict, balances_by_output: &mut HashMap<u32, BalanceSheet>, balances: &BalanceSheet, outs: &Vec<TxOut>) -> Result<()> {
      if edict.id.block == 0 && edict.id.tx != 0 {
        Err(anyhow!("invalid edict"))
      } else {
        Ok(())
      }
    }
    pub fn process_edicts(edicts: &Vec<Edict>, balances_by_output: &mut HashMap<u32, BalanceSheet>, balances: &BalanceSheet, outs: &Vec<TxOut>) -> Result<()> {
      for edict in edicts {
        Self::process_edict(edict, balances_by_output, balances, outs)?
      }
      Ok(())
    }
    pub fn handle_leftover_runes(balances: &BalanceSheet, balances_by_output: &mut HashMap<u32, BalanceSheet>) -> Result<()> {
      Ok(())
    }
    pub fn index_etching(etching: &Etching, index: u32, height: u64) -> Result<()> {
        if etching.rune.is_none() {
            return Ok(());
        }
        let name: u128;
        name = etching.rune.unwrap().0;
        //Self::get_reserved_name(height, index, name);
        let rune_id = Self::build_rune_id(height, index);
        tables::RUNES
            .RUNE_ID_TO_ETCHING
            .select(&rune_id.clone())
            .set(Arc::new(name.to_string().into_bytes()));
        tables::RUNES
            .ETCHING_TO_RUNE_ID
            .select(&name.to_string().into_bytes())
            .set(rune_id.clone());
        tables::RUNES
            .RUNE_ID_TO_HEIGHT
            .select(&rune_id.clone())
            .set_value(height);

        if let Some(divisibility) = etching.divisibility {
            tables::RUNES
                .DIVISIBILITY
                .select(&name.to_string().into_bytes())
                .set_value(divisibility);
        }
        if let Some(premine) = etching.premine {
            tables::RUNES
                .PREMINE
                .select(&name.to_string().into_bytes())
                .set_value(premine);
        }
        if let Some(terms) = etching.terms {
            if let Some(amount) = terms.amount {
                tables::RUNES
                    .AMOUNT
                    .select(&name.to_string().into_bytes())
                    .set_value(amount);
            }
            if let Some(cap) = terms.cap {
                tables::RUNES
                    .CAP
                    .select(&name.to_string().into_bytes())
                    .set_value(cap);
            }
            if let (Some(height_start), Some(height_end)) = (terms.height.0, terms.height.1) {
                tables::RUNES
                    .HEIGHTSTART
                    .select(&name.to_string().into_bytes())
                    .set_value(height_start);

                tables::RUNES
                    .HEIGHTEND
                    .select(&name.to_string().into_bytes())
                    .set_value(height_end);
            }
            if let (Some(offset_start), Some(offset_end)) = (terms.offset.0, terms.offset.1) {
                tables::RUNES
                    .OFFSETSTART
                    .select(&name.to_string().into_bytes())
                    .set_value(offset_start);
                tables::RUNES
                    .OFFSETEND
                    .select(&name.to_string().into_bytes())
                    .set_value(offset_end);
            }
        }
        if let Some(symbol) = etching.symbol {
            tables::RUNES
                .SYMBOL
                .select(&name.to_string().into_bytes())
                .set(Arc::new(symbol.to_string().into_bytes()));
        }

        if let Some(spacers) = etching.spacers {
            tables::RUNES
                .SYMBOL
                .select(&name.to_string().into_bytes())
                .set_value(spacers);
        }

        tables::RUNES
            .ETCHINGS
            .select(&name.to_string().into_bytes())
            .append(Arc::new(name.to_string().into_bytes()));
        Ok(())
    }

    // fn get_reserved_name(height: u32, index: u32, name: u128) -> Result<Vec<u8>, Error> {
    //     let mut interval: i64 =
    //         ((height - constants::GENESIS) as i64) / (constants::HEIGHT_INTERVAL as i64);
    //     let mut minimum_name: u128 = constants::MINIMUM_NAME;
    //     while interval > 0 {
    //         minimum_name = minimum_name.sub(1) / constants::TWENTY_SIX;
    //         interval -= 1;
    //     }
    //     if name < minimum_name || name >= constants::RESERVED_NAME {
    //         Ok(Vec::new())
    //     } else {
    //         Error::new("No reserve name")
    //     }
    // }

    pub fn build_rune_id(height: u64, tx: u32) -> Arc<Vec<u8>> {
        let rune_id = RuneId::new(height, tx).unwrap().to_string().into_bytes();
        return Arc::new(rune_id);
    }

    pub fn index_unspendables<T: MessageContext>(block: &Block, height: u64) -> Result<()> {
        for (index, tx) in block.txdata.iter().enumerate() {
            if let Some(Artifact::Runestone(ref runestone)) = Runestone::decipher(tx) {
                Self::index_runestone::<T>(tx, runestone, height, index as u32)?;
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
                    let outpoint_bytes: Vec<u8> = consensus_encode(&outpoint)?;
                    let address = Address::from_script(&output_script, constants::NETWORK)?;
                    tables::OUTPOINTS_FOR_ADDRESS
                        .select(&address.to_string().into_bytes())
                        .append(Arc::new(outpoint_bytes.clone()));
                    tables::OUTPOINT_SPENDABLE_BY
                        .select(&outpoint_bytes.clone())
                        .set(Arc::new(address.to_string().into_bytes()));
                }
            }
        }
        Ok(())
    }

    pub fn index_transaction_ids(block: &Block, height: u64) -> Result<()> {
        let ptr = tables::RUNES
            .HEIGHT_TO_TRANSACTION_IDS
            .select_value::<u64>(height);
        for tx in &block.txdata {
            ptr.append(Arc::new(tx.txid().as_byte_array().to_vec()));
        }
        Ok(())
    }
    pub fn index_outpoints(block: &Block, height: u64) -> Result<()> {
        for tx in &block.txdata {
            let ptr = tables::RUNES
                .OUTPOINT_TO_HEIGHT
                .select(&tx.txid().as_byte_array().to_vec());
            for i in 0..tx.output.len() {
                ptr.select_value(i as u32).set_value(height);
            }
        }
        Ok(())
    }

    pub fn index_protostones(
        tx: &Transaction,
        runestone: &Runestone,
        vout_default: u32,
    ) -> Result<()> {
        let edicts = runestone.edicts.clone();
        let protostones = Protostone::from_runestone(tx, runestone)?;
        let burns = Protostones::burns(protostones);

        Ok(())
    }

    pub fn index_block<T: MessageContext>(block: Block, height: u64) -> Result<()> {
        initialized_protocol_index().map_err(|e| anyhow!(e.to_string()))?;
        add_to_indexable_protocols(T::protocol_tag()).map_err(|e| anyhow!(e.to_string()))?;
        tables::RUNES
            .HEIGHT_TO_BLOCKHASH
            .select_value::<u64>(height)
            .set(Arc::new(block.block_hash().as_byte_array().to_vec()));
        tables::RUNES
            .BLOCKHASH_TO_HEIGHT
            .select(&block.block_hash().as_byte_array().to_vec())
            .set_value::<u64>(height);
        Self::index_spendables(&block.txdata)?;
        Self::index_transaction_ids(&block, height)?;
        Self::index_outpoints(&block, height)?;
        Self::index_unspendables::<T>(&block, height)?;
        println!("got block");
        flush();
        Ok(())
    }
}

// GENESIS RUNE REF

//     const name = nameToArrayBuffer("UNCOMMONGOODS");
//     const spacers = 128;
//     const runeId = new RuneId(1, 0).toBytes();
//     ETCHING_TO_RUNE_ID.select(name).set(runeId);
//     RUNE_ID_TO_ETCHING.select(runeId).set(name);
//     RUNE_ID_TO_HEIGHT.select(runeId).setValue<u32>(GENESIS);
//     DIVISIBILITY.select(name).setValue<u8>(1);
//     AMOUNT.select(name).set(toArrayBuffer(u128.from(1)));
//     CAP.select(name).set(toArrayBuffer(u128.Max));
//     MINTS_REMAINING.select(name).set(toArrayBuffer(u128.Max));
//     OFFSETEND.select(name).setValue<u64>(SUBSIDY_HALVING_INTERVAL);
//     SPACERS.select(name).setValue<u32>(128);
//     SYMBOL.select(name).setValue<u8>(<u8>"\u{29C9}".charCodeAt(0));
//     ETCHINGS.append(name);
//   }
