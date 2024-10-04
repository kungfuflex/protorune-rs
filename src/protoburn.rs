use anyhow::{anyhow, Result};
use std::{
    cmp::min,
    collections::{HashMap, HashSet},
    ops::Deref,
};

use ordinals::{RuneId, Runestone};

use crate::balance_sheet::BalanceSheet;

#[derive(Clone)]
pub struct Protoburn {
    pub tag: Option<u128>,
    pub pointer: Option<u32>,
    pub from: Option<Vec<u32>>,
}

impl Protoburn {}

pub trait Protoburns<T>: Deref<Target = [T]> {
    fn construct_burncycle(&self) -> Result<BurnCycle> {
        let length = u32::try_from(self.len())?;
        Ok(BurnCycle::new(length))
    }
    fn process(&mut self, runestone: &Runestone, runestone_output_index: u32) -> Result<()>;
}

impl Protoburns<Protoburn> for Vec<Protoburn> {
    fn process(&mut self, runestone: &Runestone, runestone_output_index: u32) -> Result<()> {
        //TODO: pipe stuff into runestone_balance_sheet
        let burn_cycles = self.construct_burncycle()?;
        let edicts = runestone.edicts.clone();
        let mut runestone_balance_sheet = BalanceSheet::new();
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
                        runestone_balance_sheet.decrease(rune.clone().into(), to_apply);
                        burn_sheets[i].increase(rune.into(), to_apply);
                    }
                }
            }
        }

        //TODO: process cycles
        Ok(())
    }
}

struct BurnCycle {
    max: u32,
    cycles: HashMap<RuneId, i32>,
}

impl BurnCycle {
    pub fn new(max: u32) -> Self {
        BurnCycle {
            max,
            cycles: HashMap::<RuneId, i32>::new(),
        }
    }
    pub fn next(&mut self, rune: &RuneId) -> Result<i32> {
        if !self.cycles.contains_key(rune) {
            self.cycles.insert(rune.clone(), 0);
        }
        let cycles = self.cycles.clone();
        let cycle = cycles.get(rune).ok_or(anyhow!("no value found"))?;
        self.cycles
            .insert(rune.clone(), (cycle.clone() + 1) % (self.max as i32));
        Ok(cycle.clone())
    }
    pub fn peek(&mut self, rune: &RuneId) -> Result<i32> {
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
