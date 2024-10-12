use std::collections::HashMap;

use crate::{
    balance_sheet::{BalanceSheet, ProtoruneRuneId}
};
use anyhow::{anyhow, Result};

#[derive(Clone, Copy, Default)]
pub struct RuneTransfer {
    pub id: ProtoruneRuneId,
    pub value: u128,
}

impl RuneTransfer {
    pub fn from_balance_sheet(s: BalanceSheet) -> Vec<Self> {
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
        balances_by_output: &mut HashMap<u32, BalanceSheet>,
        vout: u32,
        pointer: u32
    ) -> Result<()>;
}

impl OutgoingRunes for (Vec<RuneTransfer>, BalanceSheet) {
    fn reconcile(
        &self,
        balances_by_output: &mut HashMap<u32, BalanceSheet>,
        vout: u32,
        pointer: u32
    ) -> Result<()> {
        let runtime_initial = balances_by_output
            .get(&u32::MAX)
            .map(|_| BalanceSheet::default())
            .unwrap_or_else(|| BalanceSheet::default());
        let incoming_initial = balances_by_output
            .get(&vout)
            .ok_or("")
            .map_err(|_| anyhow!("balance sheet not found"))?
            .clone();
        let mut initial = BalanceSheet::merge(&incoming_initial, &runtime_initial);
        let outgoing: BalanceSheet = self.0.clone().into();
        initial.debit(&outgoing)?;
        self.1.clone().debit(&initial)?;
        balances_by_output.insert(u32::MAX, self.1.clone());
        balances_by_output.insert(pointer, initial);
        Ok(())
    }
}
