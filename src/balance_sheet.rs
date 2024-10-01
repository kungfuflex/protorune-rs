use metashrew_rs::index_pointer::{IndexPointer, KeyValuePointer};
use ordinals::RuneId;
use std::collections::HashMap;
use std::ops::Deref;
use std::sync::Arc;
use std::{fmt, u128};

#[derive(Eq, PartialEq, Hash, Clone, Copy)]
pub struct ProtoruneRuneId(RuneId);

impl ProtoruneRuneId {
    pub fn new(inner: RuneId) -> Self {
        ProtoruneRuneId(inner)
    }
}

impl fmt::Display for ProtoruneRuneId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "RuneId {{ block: {}, tx: {} }}", self.block, self.tx)
    }
}

impl Deref for ProtoruneRuneId {
    type Target = RuneId;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<ProtoruneRuneId> for Arc<Vec<u8>> {
    fn from(rune_id: ProtoruneRuneId) -> Self {
        let mut bytes = Vec::new();

        bytes.extend(&rune_id.block.to_le_bytes());
        bytes.extend(&rune_id.tx.to_le_bytes());

        // Wrap the Vec in an Arc
        Arc::new(bytes)
    }
}

impl From<Arc<Vec<u8>>> for ProtoruneRuneId {
    fn from(arc_bytes: Arc<Vec<u8>>) -> Self {
        // Convert the Arc<Vec<u8>> to a slice of bytes
        let bytes: &[u8] = arc_bytes.as_ref();

        // Extract the u32 and u64 from the byte slice
        let block = u64::from_le_bytes(bytes[0..8].try_into().unwrap());
        let tx = u32::from_le_bytes(bytes[8..12].try_into().unwrap());

        // Return the deserialized MyStruct
        ProtoruneRuneId {
            0: RuneId { block, tx },
        }
    }
}

pub struct BalanceSheet {
    balances: HashMap<ProtoruneRuneId, u128>, // Using HashMap to map runes to their balances
}

impl BalanceSheet {
    pub fn new() -> Self {
        BalanceSheet {
            balances: HashMap::new(),
        }
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

    pub fn set(&mut self, rune: ProtoruneRuneId, value: u128) {
        self.balances.insert(rune, value);
    }

    pub fn increase(&mut self, rune: ProtoruneRuneId, value: u128) {
        let current_balance = self.get(&rune);
        self.set(rune, current_balance + value);
    }

    pub fn decrease(&mut self, rune: ProtoruneRuneId, value: u128) -> bool {
        let current_balance = self.get(&rune);
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
            merged.set(rune.clone(), *balance);
        }
        for (rune, balance) in &b.balances {
            let current_balance = merged.get(rune);
            merged.set(rune.clone(), current_balance + *balance);
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

    pub fn save(&self, ptr: &IndexPointer, is_cenotaph: bool) {
        let runes_ptr = ptr.keyword("/runes");
        let balances_ptr = ptr.keyword("/balances");

        for (rune, balance) in &self.balances {
            if *balance != 0u128 && !is_cenotaph {
                runes_ptr.append((*rune).into());

                balances_ptr.append_value::<u128>(*balance);
            }
        }
    }

    pub fn load(ptr: &IndexPointer) -> BalanceSheet {
        let runes_ptr = ptr.keyword("/runes");
        let balances_ptr = ptr.keyword("/balances");
        let length = runes_ptr.length_key().get_value::<u32>();
        let mut result = BalanceSheet::new();

        for i in 0..length {
            let rune = ProtoruneRuneId::from(runes_ptr.select_index(i).get());
            let balance = balances_ptr.select_index(i).get_value::<u128>();
            result.set(rune, balance);
        }
        result
    }
}
