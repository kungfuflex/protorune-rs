use crate::rune_transfer::RuneTransfer;
use anyhow::{anyhow, Result};
#[allow(unused_imports)]
use ordinals::RuneId;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::{fmt, u128};

#[derive(
    Eq, PartialOrd, Ord, PartialEq, Hash, Clone, Copy, Debug, Default, Serialize, Deserialize,
)]
pub struct ProtoruneRuneId {
    pub block: u128,
    pub tx: u128,
}

pub trait RuneIdentifier {
    fn to_pair(&self) -> (u128, u128);
}

impl ProtoruneRuneId {
    pub fn new(block: u128, tx: u128) -> Self {
        ProtoruneRuneId { block, tx }
    }
    pub fn delta(self, next: ProtoruneRuneId) -> Option<(u128, u128)> {
        let block = next.block.checked_sub(self.block)?;

        let tx = if block == 0 {
            next.tx.checked_sub(self.tx)?
        } else {
            next.tx
        };

        Some((block.into(), tx.into()))
    }
}

impl RuneIdentifier for ProtoruneRuneId {
    fn to_pair(&self) -> (u128, u128) {
        return (self.block, self.tx);
    }
}

impl RuneIdentifier for RuneId {
    fn to_pair(&self) -> (u128, u128) {
        return (self.block as u128, self.tx as u128);
    }
}

impl From<RuneId> for ProtoruneRuneId {
    fn from(v: RuneId) -> ProtoruneRuneId {
        let (block, tx) = v.to_pair();
        ProtoruneRuneId::new(block as u128, tx as u128)
    }
}

impl fmt::Display for ProtoruneRuneId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "RuneId {{ block: {}, tx: {} }}", self.block, self.tx)
    }
}

impl From<ProtoruneRuneId> for Vec<u8> {
    fn from(rune_id: ProtoruneRuneId) -> Self {
        let mut bytes = Vec::new();
        let (block, tx) = rune_id.to_pair();

        bytes.extend(&block.to_le_bytes());
        bytes.extend(&tx.to_le_bytes());
        bytes
    }
}

impl From<ProtoruneRuneId> for Arc<Vec<u8>> {
    fn from(rune_id: ProtoruneRuneId) -> Self {
        let bytes = rune_id.into();
        // Wrap the Vec in an Arc
        Arc::new(bytes)
    }
}

impl From<Arc<Vec<u8>>> for ProtoruneRuneId {
    fn from(arc_bytes: Arc<Vec<u8>>) -> Self {
        // Convert the Arc<Vec<u8>> to a slice of bytes
        let bytes: &[u8] = arc_bytes.as_ref();

        // Extract the u32 and u64 from the byte slice
        let block = u128::from_le_bytes((&bytes[0..16]).try_into().unwrap());
        let tx = u128::from_le_bytes((&bytes[16..32]).try_into().unwrap());

        // Return the deserialized MyStruct
        ProtoruneRuneId { block, tx }
    }
}

#[derive(Clone, Default, Debug, Serialize, Deserialize)]
pub struct BalanceSheet {
    pub balances: HashMap<ProtoruneRuneId, u128>, // Using HashMap to map runes to their balances
}

impl BalanceSheet {
    pub fn new() -> Self {
        BalanceSheet {
            balances: HashMap::new(),
        }
    }

    pub fn from_pairs(runes: Vec<ProtoruneRuneId>, balances: Vec<u128>) -> BalanceSheet {
        let mut sheet = BalanceSheet::new();
        for i in 0..runes.len() {
            sheet.set(&runes[i], balances[i]);
        }
        return sheet;
    }

    pub fn pipe(&self, sheet: &mut BalanceSheet) -> () {
        for (rune, balance) in &self.balances {
            sheet.increase(rune, *balance);
        }
    }
    pub fn debit(&mut self, sheet: &BalanceSheet) -> Result<()> {
        for (rune, balance) in &sheet.balances {
            if sheet.get(&rune) > self.get(&rune) {
                return Err(anyhow!("balance underflow"));
            }
            self.decrease(rune, *balance);
        }
        Ok(())
    }

    pub fn inspect(&self) -> String {
        let mut base = String::from("balances: [\n");
        for (rune, balance) in &self.balances {
            base.push_str(&format!("  {}: {}\n", rune, balance));
        }
        base.push_str("]");
        base
    }

    pub fn get(&self, rune: &ProtoruneRuneId) -> u128 {
        *self.balances.get(rune).unwrap_or(&0u128) // Return 0 if rune not found
    }

    pub fn set(&mut self, rune: &ProtoruneRuneId, value: u128) {
        self.balances.insert(rune.clone(), value);
    }

    pub fn increase(&mut self, rune: &ProtoruneRuneId, value: u128) {
        let current_balance = self.get(rune);
        self.set(rune, current_balance + value);
    }

    pub fn decrease(&mut self, rune: &ProtoruneRuneId, value: u128) -> bool {
        let current_balance = self.get(rune);
        if current_balance < value {
            false
        } else {
            self.set(rune, current_balance - value);
            true
        }
    }

    pub fn merge(a: &BalanceSheet, b: &BalanceSheet) -> BalanceSheet {
        let mut merged = BalanceSheet::new();
        for (rune, balance) in &a.balances {
            merged.set(rune, *balance);
        }
        for (rune, balance) in &b.balances {
            let current_balance = merged.get(rune);
            merged.set(rune, current_balance + *balance);
        }
        merged
    }

    pub fn concat(ary: Vec<BalanceSheet>) -> BalanceSheet {
        let mut concatenated = BalanceSheet::new();
        for sheet in ary {
            concatenated = BalanceSheet::merge(&concatenated, &sheet);
        }
        concatenated
    }
}

impl PartialEq for BalanceSheet {
    fn eq(&self, other: &Self) -> bool {
        self.balances == other.balances
    }
}

impl From<Vec<RuneTransfer>> for BalanceSheet {
    fn from(v: Vec<RuneTransfer>) -> BalanceSheet {
        BalanceSheet {
            balances: HashMap::<ProtoruneRuneId, u128>::from_iter(
                v.into_iter().map(|v| (v.id, v.value)),
            ),
        }
    }
}

pub trait IntoString {
    fn to_str(&self) -> String;
}

impl IntoString for Vec<u8> {
    fn to_str(&self) -> String {
        let mut str: String = "".to_string();
        for i in self {
            str = str + i.to_string().as_str() + ",";
            ()
        }

        str
    }
}
