use crate::{
    balance_sheet::{BalanceSheet, ProtoruneRuneId},
    tables::RuneTable,
};
use anyhow::Result;
use metashrew::index_pointer::AtomicPointer;

#[derive(Clone, Default)]
pub struct RuneTransfer {
    pub id: ProtoruneRuneId,
    pub value: u128,
}

impl RuneTransfer {
    pub fn from_balance_sheet(s: BalanceSheet, tag: u128, atomic: &mut AtomicPointer) -> Vec<Self> {
        s.balances
            .iter()
            .map(|(id, v)| Self {
                id: id.clone(),
                value: *v
            })
            .collect::<Vec<RuneTransfer>>()
    }
}
