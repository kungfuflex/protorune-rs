use std::collections::HashMap;

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
                value: *v,
            })
            .collect::<Vec<RuneTransfer>>()
    }
}

pub trait OutgoingRunes {
    fn reconcile(
        &self,
        initial_sheet: BalanceSheet,
        pointer: u32,
        refund_pointer: u32,
    ) -> Result<HashMap<u32, BalanceSheet>>;
}

impl OutgoingRunes for (Vec<RuneTransfer>, BalanceSheet) {
    fn reconcile(
        &self,
        _initial_sheet: BalanceSheet,
        pointer: u32,
        refund_pointer: u32,
    ) -> Result<HashMap<u32, BalanceSheet>> {
        let mut balances_by_output = HashMap::<u32, BalanceSheet>::new();
        let mut initial_sheet = _initial_sheet.clone();
        let mut sheet = BalanceSheet::default();
        self.clone()
            .0
            .into_iter()
            .map(|transfer| {
                sheet.increase(transfer.id, transfer.value);
                initial_sheet.decrease(transfer.id, transfer.value);
            })
            .for_each(drop);
        balances_by_output.insert(pointer, sheet);
        self.clone()
            .1
            .balances
            .into_iter()
            .map(|(id, value)| {
                initial_sheet.decrease(id, value);
            })
            .for_each(drop);
        balances_by_output.insert(u32::MAX, self.clone().1);
        balances_by_output.insert(refund_pointer, initial_sheet);
        Ok(balances_by_output)
    }
}
