use crate::message::MessageContext;
use anyhow::{ anyhow, Ok, Result };
use bitcoin::blockdata::block::Block;
use bitcoin::consensus::encode::serialize;
use bitcoin::hashes::Hash;
use bitcoin::{ block, Address, OutPoint, Script, ScriptBuf, Transaction };
use metashrew::index_pointer::KeyValuePointer;
use metashrew::{ flush, println, stdout };
use ordinals::Etching;
use ordinals::{ Artifact, Runestone };
use protostone::{ add_to_indexable_protocols, initialized_protocol_index };
use std::fmt::Write;
use std::ops::Sub;
use std::sync::Arc;

pub mod balance_sheet;
pub mod byte_utils;
pub mod constants;
pub mod message;
pub mod protoburn;
pub mod protostone;
pub mod view;
#[cfg(test)]
pub mod tests;

pub struct Protorune(());

impl Protorune {
    pub fn index_runestone<T: MessageContext>(
        runestone: &Runestone,
        height: u32,
        index: u32
    ) -> Result<()> {
        if let Some(etching) = runestone.etching.as_ref() {
            Self::index_etching(etching, index, height)?;
        }
        Ok(())
    }
    pub fn index_etching(etching: &Etching, index: u32, height: u32) -> Result<()> {
        if etching.rune.is_none() {
            return Ok(());
        }
        let name: u128;
        name = etching.rune.unwrap().0;
        Self::get_reserved_name(height, index, name);

        Ok(())
    }

    fn get_reserved_name(height: u32, index: u32, name: u128) {
        let mut interval: i64 =
            ((height - constants::GENESIS) as i64) / (constants::HEIGHT_INTERVAL as i64);
        let mut minimum_name: u128 = constants::MINIMUM_NAME;
        while interval > 0 {
            minimum_name = minimum_name.sub(1) / constants::TWENTY_SIX;
            interval -= 1;
        }
        if name {
            if name < minimum_name || name >= constants::RESERVED_NAME {
            }
        }
    }

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

    //   buildRuneId(height: u64, tx: u32): ArrayBuffer {
    //     const runeId = new RuneId(height, tx).toBytes();
    //     HEIGHT_TO_RUNE_IDS.selectValue<u32>(<u32>height).append(runeId);
    //     return runeId;
    //   }

    //       etch(
    //     height: u64,
    //     tx: u32,
    //     initialBalanceSheet: BalanceSheet,
    //     transaction: RunesTransaction,
    //   ): bool {
    //     if (!this.isEtching()) return false;
    //     const name = this.getReservedNameFor(height, tx);
    //     if (isNullPtr<ArrayBuffer>(name)) return false;
    //     if (ETCHING_TO_RUNE_ID.select(name).get().byteLength !== 0) return false; // already taken / commitment not foun
    //     const runeId = this.buildRuneId(height, tx);
    //     RUNE_ID_TO_ETCHING.select(runeId).set(name);
    //     ETCHING_TO_RUNE_ID.select(name).set(runeId);
    //     RUNE_ID_TO_HEIGHT.select(runeId).setValue<u32>(<u32>height);
    //     if (this.fields.has(Field.DIVISIBILITY))
    //       DIVISIBILITY.select(name).setValue<u8>(
    //         fieldTo<u8>(this.fields.get(Field.DIVISIBILITY)),
    //       );
    //     if (this.fields.has(Field.PREMINE)) {
    //       const premine = fieldToU128(this.fields.get(Field.PREMINE));
    //       BalanceSheet.fromPairs([runeId], [premine]).pipe(initialBalanceSheet);
    //       PREMINE.select(name).set(toArrayBuffer(premine));
    //     }
    //     if (this.getFlag(Flag.TERMS)) {
    //       if (this.fields.has(Field.AMOUNT))
    //         AMOUNT.select(name).set(
    //           toArrayBuffer(fieldToU128(this.fields.get(Field.AMOUNT))),
    //         );

    //       if (this.fields.has(Field.CAP)) {
    //         CAP.select(name).set(
    //           toArrayBuffer(fieldToU128(this.fields.get(Field.CAP))),
    //         );
    //         MINTS_REMAINING.select(name).set(
    //           fieldToArrayBuffer(this.fields.get(Field.CAP)),
    //         );
    //       }
    //       if (this.fields.has(Field.HEIGHTSTART))
    //         HEIGHTSTART.select(name).setValue<u64>(
    //           fieldTo<u64>(this.fields.get(Field.HEIGHTSTART)),
    //         );
    //       if (this.fields.has(Field.HEIGHTEND))
    //         HEIGHTEND.select(name).setValue<u64>(
    //           fieldTo<u64>(this.fields.get(Field.HEIGHTEND)),
    //         );
    //       if (this.fields.has(Field.OFFSETSTART))
    //         OFFSETSTART.select(name).setValue<u64>(
    //           fieldTo<u64>(this.fields.get(Field.OFFSETSTART)),
    //         );
    //       if (this.fields.has(Field.OFFSETEND))
    //         OFFSETEND.select(name).setValue<u64>(
    //           fieldTo<u64>(this.fields.get(Field.OFFSETEND)),
    //         );
    //     }
    //     if (this.fields.has(Field.SPACERS))
    //       SPACERS.select(name).setValue<u32>(
    //         fieldTo<u32>(this.fields.get(Field.SPACERS)),
    //       );
    //     if (this.fields.has(Field.SYMBOL))
    //       SYMBOL.select(name).setValue<u8>(
    //         fieldTo<u8>(this.fields.get(Field.SYMBOL)),
    //       );
    //     ETCHINGS.append(name);
    //     return true;
    //   }

    pub fn index_unspendables<T: MessageContext>(block: &Block, height: u32) -> Result<()> {
        for (index, tx) in block.txdata.iter().enumerate() {
            if let Some(Artifact::Runestone(ref runestone)) = Runestone::decipher(tx) {
                Self::index_runestone::<T>(tx, runestone, height, index)?;
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
