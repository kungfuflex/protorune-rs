use metashrew_rs::index_pointer::IndexPointer;
use std::collections::HashMap;
use std::u128;

pub struct BalanceSheet {
    balances: HashMap<String, u128>, // Using HashMap to map runes to their balances
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

    pub fn get(&self, rune: &str) -> u128 {
        *self.balances.get(rune).unwrap_or(&(u128::from(0))) // Return 0 if rune not found
    }

    pub fn set(&mut self, rune: String, value: u128) {
        self.balances.insert(rune, value);
    }

    pub fn increase(&mut self, rune: String, value: u128) {
        let current_balance = self.get(&rune);
        self.set(rune, current_balance + value);
    }

    pub fn decrease(&mut self, rune: String, value: u128) -> bool {
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
            if *balance != u128::from(0) && !is_cenotaph {
                runes_ptr.append(rune.as_bytes()); // Convert String to &[u8]

                let buf: Vec<u8> = balance.to_bytes(); // You need to implement this method
                balances_ptr.append(buf.as_slice());
            }
        }
    }

    pub fn load(ptr: &IndexPointer) -> BalanceSheet {
        let runes_ptr = ptr.keyword("/runes");
        let balances_ptr = ptr.keyword("/balances");
        let length = runes_ptr.length_key().get_value::<u32>();
        let mut result = BalanceSheet::new();

        for i in 0..length {
            let rune = String::from_utf8_lossy(&runes_ptr.select_index(i).get()).to_string(); // Convert &[u8] to String
            let balance = from_array_buffer(balances_ptr.select_index(i).get());
            result.set(rune, balance);
        }
        result
    }
}
