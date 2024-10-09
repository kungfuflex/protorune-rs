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
        balances_by_output: &mut HashMap<u32, BalanceSheet>,
        vout: u32,
        pointer: u32
        refund_pointer: u32
    ) -> Result<()> {
        let mut runtime_initial = balances_by_output.get(u32::MAX).unwrap_or_else(|| BalanceSheet::default());
        let mut incoming_initial = balances_by_output.get(vout).clone();
        let mut initial = BalanceSheet::merge(&incoming_initial, &runtime_initial);
        let outgoing: BalanceSheet = self.0.into();
        initial.debit(&outgoing)?;
        self.1.clone().debit(&initial)?;
        balances_by_output.insert(u32::MAX, self.1);
        balances_by_output.insert(pointer, &initial);   
        Ok(())
    }
}
