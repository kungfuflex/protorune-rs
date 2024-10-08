use crate::{
    balance_sheet::{BalanceSheet, ProtoruneRuneId},
    tables::RuneTable,
};
use anyhow::Result;
use metashrew::index_pointer::AtomicPointer;

#[derive(Clone, Default)]
pub struct IncomingRune {
    pub rune: ProtoruneRuneId,
    pub amount: u128,
    deposit_amount: u128,
    initial_amount: u128,
    pointer_index: i32,
    refund_pointer_index: i32,
    outpoint_index: i32,
    atomic: AtomicPointer,
    table: RuneTable,
}

impl IncomingRune {
    pub fn from_balance_sheet(s: BalanceSheet, tag: u128, atomic: &mut AtomicPointer) -> Vec<Self> {
        s.balances
            .iter()
            .map(|(id, v)| Self {
                rune: id.clone(),
                amount: *v,
                deposit_amount: 0,
                initial_amount: *v,
                pointer_index: -1,
                refund_pointer_index: -1,
                outpoint_index: -1,
                atomic: atomic.clone(),
                table: RuneTable::for_protocol(tag),
            })
            .collect::<Vec<IncomingRune>>()
    }
    /* -------------------------
      TODO: Implement all the base functions
    ----------------------------- */
    pub fn refund(&self, amount: u128) -> Result<()> {
        /*
         * TODO: implement logic
        self.context
            .table
            .OUTPOINT_TO_RUNES
            .select(&self.context.refund_pointer.try_to_bytes()?)
            .get();
        */
        Ok(())
    }

    pub fn refund_deposit(&self, amount: u128) -> Result<()> {
        Ok(())
    }

    pub fn refund_all(&self) -> Result<()> {
        self.refund(self.initial_amount - self.amount)?;
        self.refund_deposit(self.deposit_amount)?;
        Ok(())
    }
    pub fn forward(&self, amount: u128) -> Result<()> {
        Ok(())
    }
    pub fn forward_all(&self) -> Result<()> {
        self.forward(self.amount)?;
        Ok(())
    }
    pub fn deposit(&self, amount: u128) -> Result<()> {
        Ok(())
    }
    pub fn deposit_all(&self) -> Result<()> {
        self.deposit(self.amount)?;
        Ok(())
    }
}
