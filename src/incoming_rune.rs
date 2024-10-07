use crate::{
    balance_sheet::{ProtoruneRuneId, BalanceSheet},
    message::{MessageContextParcel, ToBytes},
};
use anyhow::{anyhow, Result};
use metashrew::index_pointer::KeyValuePointer;
use std::sync::Arc;

#[derive(Clone, Default)]
pub struct IncomingRune {
    pub rune: ProtoruneRuneId,
    pub amount: u128,
    deposit_amount: u128,
    initial_amount: u128,
    pointer_index: i32,
    refund_pointer_index: i32,
    outpoint_index: i32,
    context: Arc<MessageContextParcel>,
}

impl From<BalanceSheet> for Vec<IncomingRune> {
  fn from(v: BalanceSheet) -> Vec<IncomingRune> {
    v.balances.iter().map(|(id, v)| {
      IncomingRune {
        rune: id.clone(),
        amount: *v,
        deposit_amount: 0,
        initial_amount: *v,
        pointer_index: 0,
        refund_pointer_index: 0,
        outpoint_index: 0,
        context: Arc::new(MessageContextParcel::default())
      }
    }).collect::<Vec<IncomingRune>>()
  }
  
}

impl IncomingRune {
    pub fn from_message(
        rune: ProtoruneRuneId,
        amount: u128,
        parcel: Arc<MessageContextParcel>,
    ) -> Self {
        Self {
            context: parcel.clone(),
            rune,
            amount,
            deposit_amount: 0,
            initial_amount: amount,
            pointer_index: -1,
            refund_pointer_index: -1,
            outpoint_index: -1,
        }
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
