use anyhow::{ anyhow, Result };
use bitcoin::consensus::Encodable;
use metashrew::{
    index_pointer::{ AtomicPointer, IndexPointer, KeyValuePointer },
    println,
    stdio::stdout,
};
use serde::{ Deserialize, Serialize };
use crate::{ proto, rune_transfer::RuneTransfer, tables, utils::consensus_encode };
use ordinals::RuneId;
use std::{ collections::HashMap, sync::atomic };
use std::fmt::Write;
use std::sync::Arc;
use std::{ fmt, u128 };
use protobuf::{ Message, MessageField };

#[derive(Eq, PartialEq, Hash, Clone, Copy, Debug, Default, Serialize, Deserialize)]
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
        let block = u64::from_le_bytes(bytes[0..8].try_into().unwrap());
        let tx = u32::from_le_bytes(bytes[8..12].try_into().unwrap());

        // Return the deserialized MyStruct
        (RuneId { block, tx }).into()
    }
}

#[derive(Clone, Default, Debug, Serialize, Deserialize)]
pub struct BalanceSheet {
    pub balances: HashMap<ProtoruneRuneId, u128>,
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

    pub fn save<T: KeyValuePointer>(&self, ptr: &T, is_cenotaph: bool) {
        let runes_ptr = ptr.keyword("/runes");
        let balances_ptr = ptr.keyword("/balances");

        for (rune, balance) in &self.balances {
            if *balance != 0u128 && !is_cenotaph {
                runes_ptr.append((*rune).into());

                balances_ptr.append_value::<u128>(*balance);
            }
        }
    }

    pub fn save_index<T: KeyValuePointer>(
        &self,
        rune: &ProtoruneRuneId,
        ptr: &T,
        is_cenotaph: bool
    ) -> Result<()> {
        let runes_ptr = ptr.keyword("/runes");
        let balances_ptr = ptr.keyword("/balances");
        let balance = self.balances.get(rune).ok_or(anyhow!("no balance found"))?;
        if *balance != 0u128 && !is_cenotaph {
            runes_ptr.append((*rune).into());
            balances_ptr.append_value::<u128>(*balance);
        }

        Ok(())
    }

    pub fn load<T: KeyValuePointer>(ptr: &T) -> BalanceSheet {
        let runes_ptr = ptr.keyword("/runes");
        let balances_ptr = ptr.keyword("/balances");
        let length = runes_ptr.length();
        let mut result = BalanceSheet::new();

        for i in 0..length {
            let rune = ProtoruneRuneId::from(runes_ptr.select_index(i).get());
            let balance = balances_ptr.select_index(i).get_value::<u128>();
            result.set(&rune, balance);
        }
        result
    }
}

impl From<Vec<RuneTransfer>> for BalanceSheet {
    fn from(v: Vec<RuneTransfer>) -> BalanceSheet {
        BalanceSheet {
            balances: HashMap::<ProtoruneRuneId, u128>::from_iter(
                v.into_iter().map(|v| (v.id, v.value))
            ),
        }
    }
}

pub fn balance_sheet_to_protobuf(sheet: BalanceSheet) -> proto::protorune::BalanceSheet {
    let atomic: AtomicPointer = AtomicPointer::default();
    let mut balance_sheet = proto::protorune::BalanceSheet::new();

    for (protorune_id, balance) in sheet.balances.iter() {
        let name = atomic
            .derive(
                &tables::RUNES.RUNE_ID_TO_ETCHING.select(&consensus_encode(protorune_id).unwrap())
            )
            .get();
        let spacers = atomic.derive(&tables::RUNES.SPACERS.select(&name));
        let divisibility: u32 = atomic
            .derive(&tables::RUNES.DIVISIBILITY.select(&name))
            .get_value::<u8>()
            .into();

        let mut rune = proto::protorune::Rune::new();
        let mut rune_id = proto::protorune::RuneId::new();

        rune_id.height = protorune_id.block as u32;
        rune_id.txindex = protorune_id.tx as u32;

        rune.runeId = MessageField::some(rune_id);
        rune.name = name.to_vec();

        rune.divisibility = divisibility;
        rune.symbol = atomic.derive(&tables::RUNES.SYMBOL.select(&name)).get_value::<u8>() as u32;
        rune.spacers = atomic.derive(&tables::RUNES.SPACERS.select(&name)).get_value::<u32>();

        balance_sheet.entries.push(rune);
    }

    let mut entry = proto::protorune::BalanceSheetItem::new();
    entry.rune = MessageField::some(runes[i].clone());
    entry.balance = balance.clone();
    balance_sheet.entries.push(entry);

    balance_sheet
}
