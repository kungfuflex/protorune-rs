use crate::tables::{RuneTable, RUNES};
use anyhow::{anyhow, Result};
use bitcoin::{OutPoint, Txid};
use metashrew::index_pointer::{AtomicPointer, KeyValuePointer};
use std::{
    cmp::min,
    collections::{HashMap, HashSet},
    ops::Deref,
    sync::Arc,
};

use ordinals::Runestone;

use protorune_support::balance_sheet::{BalanceSheet, ProtoruneRuneId};

#[derive(Clone)]
pub struct Protoburn {
    pub tag: Option<u128>,
    pub pointer: Option<u32>,
    pub from: Option<Vec<u32>>,
}

impl Protoburn {
    pub fn process(
        &mut self,
        atomic: &mut AtomicPointer,
        balance_sheet: BalanceSheet,
        proto_balances_by_output: &mut HashMap<u32, BalanceSheet>,
        outpoint: OutPoint,
    ) -> Result<()> {
        let table = RuneTable::for_protocol(self.tag.ok_or(anyhow!("no tag found"))?);
        for (rune, _balance) in balance_sheet.clone().balances.into_iter() {
            let name = RUNES.RUNE_ID_TO_ETCHING.select(&rune.into()).get();
            let runeid: Arc<Vec<u8>> = rune.into();
            atomic
                .derive(&table.RUNE_ID_TO_ETCHING.select(&runeid))
                .set(name.clone());
            atomic
                .derive(&table.ETCHING_TO_RUNE_ID.select(&name))
                .set(runeid);
            atomic
                .derive(&table.SPACERS.select(&name))
                .set(RUNES.SPACERS.select(&name).get());
            atomic
                .derive(&table.DIVISIBILITY.select(&name))
                .set(RUNES.DIVISIBILITY.select(&name).get());
            atomic
                .derive(&table.SYMBOL.select(&name))
                .set(RUNES.SYMBOL.select(&name).get());
            atomic.derive(&table.ETCHINGS).append(name);
        }
        if !proto_balances_by_output.contains_key(&outpoint.vout) {
            proto_balances_by_output.insert(outpoint.vout, BalanceSheet::default());
        }
        balance_sheet.pipe(proto_balances_by_output.get_mut(&outpoint.vout).unwrap());
        Ok(())
    }
}

pub trait Protoburns<T>: Deref<Target = [T]> {
    fn construct_burncycle(&self) -> Result<BurnCycle> {
        let length = u32::try_from(self.len())?;
        Ok(BurnCycle::new(length))
    }
    fn process(
        &mut self,
        atomic: &mut AtomicPointer,
        runestone: &Runestone,
        runestone_output_index: u32,
        balances_by_output: &HashMap<u32, BalanceSheet>,
        proto_balances_by_output: &mut HashMap<u32, BalanceSheet>,
        default_output: u32,
        txid: Txid,
    ) -> Result<()>;
}

impl Protoburns<Protoburn> for Vec<Protoburn> {
    fn process(
        &mut self,
        atomic: &mut AtomicPointer,
        runestone: &Runestone,
        runestone_output_index: u32,
        balances_by_output: &HashMap<u32, BalanceSheet>,
        proto_balances_by_output: &mut HashMap<u32, BalanceSheet>,
        default_output: u32,
        txid: Txid,
    ) -> Result<()> {
        let mut runestone_balance_sheet = BalanceSheet::new();
        if balances_by_output.contains_key(&runestone_output_index) {
            let sheet = balances_by_output
                .get(&runestone_output_index)
                .ok_or(anyhow!("cannot find balance sheet"))?;
            sheet.pipe(&mut runestone_balance_sheet);
        }
        //TODO: pipe stuff into runestone_balance_sheet
        let mut burn_cycles = self.construct_burncycle()?;
        let edicts = runestone.edicts.clone();
        let mut pull_set = HashMap::<u32, bool>::new();
        let mut burn_sheets = self
            .into_iter()
            .map(|_a| BalanceSheet::new())
            .collect::<Vec<BalanceSheet>>();
        for (i, burn) in self.into_iter().enumerate() {
            if let Some(_from) = burn.clone().from {
                let from = _from.into_iter().collect::<HashSet<u32>>();
                for j in from {
                    pull_set.insert(j, true);
                    if edicts[j as usize].output == runestone_output_index {
                        let rune = edicts[j as usize].id;
                        let remaining = runestone_balance_sheet.get(&rune.into());
                        let to_apply = min(remaining, edicts[j as usize].amount);
                        if to_apply == 0 {
                            continue;
                        }
                        runestone_balance_sheet.decrease(&rune.clone().into(), to_apply);
                        burn_sheets[i].increase(&rune.into(), to_apply);
                    }
                }
            }
        }

        for (i, edict) in edicts.into_iter().enumerate() {
            if pull_set.contains_key(&(i as u32)) {
                continue;
            };
            if edict.output == runestone_output_index {
                let rune = edict.id;
                let cycle = burn_cycles.peek(&(rune.into()))?;
                let remaining = runestone_balance_sheet.get(&(rune.into()));
                let to_apply = min(remaining, edict.amount);
                if to_apply == 0 {
                    continue;
                };
                burn_cycles.next(&(rune.into()))?;
                runestone_balance_sheet.decrease(&rune.clone().into(), to_apply);
                burn_sheets[cycle as usize].increase(&rune.into(), to_apply);
            }
        }

        // the default output of the runestone (all leftover runes, or the mint runes go to this output)
        // equals the runestone OP_RETURN. This is a valid protoburn
        if runestone_output_index == default_output {
            for rune in runestone_balance_sheet.clone().balances.keys() {
                let cycle = burn_cycles.peek(rune)?;
                let to_apply = runestone_balance_sheet.get(rune);
                if to_apply == 0 {
                    continue;
                };
                burn_cycles.next(rune)?;
                runestone_balance_sheet.decrease(rune, to_apply);
                burn_sheets[cycle as usize].increase(rune, to_apply);
            }
        }

        for (i, burn) in self.into_iter().enumerate() {
            let sheet = burn_sheets[i].clone();
            burn.process(
                atomic,
                sheet,
                proto_balances_by_output,
                OutPoint::new(txid, burn.pointer.ok_or(anyhow!("no vout on protoburn"))?),
            )?;
        }
        Ok(())
    }
}

pub struct BurnCycle {
    max: u32,
    cycles: HashMap<ProtoruneRuneId, i32>,
}

impl BurnCycle {
    pub fn new(max: u32) -> Self {
        BurnCycle {
            max,
            cycles: HashMap::<ProtoruneRuneId, i32>::new(),
        }
    }
    pub fn next(&mut self, rune: &ProtoruneRuneId) -> Result<i32> {
        if !self.cycles.contains_key(rune) {
            self.cycles.insert(rune.clone(), 0);
        }
        let cycles = self.cycles.clone();
        let cycle = cycles.get(rune).ok_or(anyhow!("no value found"))?;
        self.cycles
            .insert(rune.clone(), (cycle.clone() + 1) % (self.max as i32));
        Ok(cycle.clone())
    }
    pub fn peek(&mut self, rune: &ProtoruneRuneId) -> Result<i32> {
        if !self.cycles.contains_key(rune) {
            self.cycles.insert(rune.clone(), 0);
        }
        Ok(self
            .cycles
            .get(rune)
            .ok_or(anyhow!("value not found"))?
            .clone())
    }
}
