use metashrew::index_pointer::{KeyValuePointer};
use protorune_support::balance_sheet::{ProtoruneRuneId, BalanceSheet};
use std::collections::{HashMap};
use anyhow::{anyhow, Result};
pub trait PersistentRecord {
    fn save<T: KeyValuePointer>(&self, ptr: &T, is_cenotaph: bool) {
        let runes_ptr = ptr.keyword("/runes");
        let balances_ptr = ptr.keyword("/balances");

        for (rune, balance) in self.balances() {
            if *balance != 0u128 && !is_cenotaph {
                runes_ptr.append((*rune).into());

                balances_ptr.append_value::<u128>(*balance);
            }
        }
    }
    fn balances(&self) -> &HashMap<ProtoruneRuneId, u128>;
    fn save_index<T: KeyValuePointer>(
        &self,
        rune: &ProtoruneRuneId,
        ptr: &T,
        is_cenotaph: bool,
    ) -> Result<()> {
        let runes_ptr = ptr.keyword("/runes");
        let balances_ptr = ptr.keyword("/balances");
        let balance = self.balances().get(rune).ok_or(anyhow!("no balance found"))?;
        if *balance != 0u128 && !is_cenotaph {
            runes_ptr.append((*rune).into());
            balances_ptr.append_value::<u128>(*balance);
        }

        Ok(())
    }
}
pub fn load_sheet<T: KeyValuePointer>(ptr: &T) -> BalanceSheet {
    let runes_ptr = ptr.keyword("/runes");
    let balances_ptr = ptr.keyword("/balances");
    let length = runes_ptr.length();
    let mut result = BalanceSheet::default();

    for i in 0..length {
        let rune = ProtoruneRuneId::from(runes_ptr.select_index(i).get());
        let balance = balances_ptr.select_index(i).get_value::<u128>();
        result.set(&rune, balance);
    }
    result
}

impl PersistentRecord for BalanceSheet {
    fn balances(&self) -> &HashMap<ProtoruneRuneId, u128> {
        &self.balances
    }
}
