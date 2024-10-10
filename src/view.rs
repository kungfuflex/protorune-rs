use crate::balance_sheet::BalanceSheet;
use crate::proto::protorune::{
    OutpointResponse,
    WalletResponse,
    Output,
    Outpoint,
    BalanceSheet as ProtoBalanceSheet,
    BalanceSheetItem,
    RuneId,
    Rune
};
use crate::tables;
use crate::utils::{consensus_encode, consensus_decode};
use bitcoin::consensus::Decodable;
use bitcoin::hashes::{ sha256d, Hash };
use bitcoin::OutPoint;
use metashrew::index_pointer::{ AtomicPointer, KeyValuePointer };
use protobuf::{ SpecialFields, Message, MessageField };
use metashrew::utils::{consume_exact, consume_sized_int};
use anyhow::{Result};
use std::io::{Cursor};
use bitcoin;

pub fn outpoint_to_bytes(outpoint: &OutPoint) -> Result<Vec<u8>> {
  Ok(consensus_encode(outpoint)?)
}

pub fn core_outpoint_to_proto(outpoint: &OutPoint) -> Outpoint {
  Outpoint {
    txid: outpoint.txid.as_byte_array().to_vec().clone(),
    vout: outpoint.vout,
    special_fields: SpecialFields::new()
  }
}

pub fn balance_sheet_to_proto(balance_sheet: &crate::balance_sheet::BalanceSheet) -> ProtoBalanceSheet {
  ProtoBalanceSheet {
    entries: balance_sheet.balances.clone().iter().map(|(k, v)| BalanceSheetItem { special_fields: SpecialFields::new(), rune: MessageField::some(Rune { special_fields: SpecialFields::new(), runeId: MessageField::some(RuneId { special_fields: SpecialFields::new(), height: k.block as u32, txindex: k.tx as u32 }), name: "name".as_bytes().to_vec(), divisibility: 1, spacers: 1, symbol: 1 }), balance: (&v.to_le_bytes()).to_vec() }).collect::<Vec<BalanceSheetItem>>(),
    special_fields: SpecialFields::new()
  }
}

pub fn runes_by_address(height: u32, address: &Vec<u8>) -> Result<WalletResponse> {
    let atomic = AtomicPointer::default();
    let outpoints = tables::OUTPOINTS_FOR_ADDRESS.select(address).get_list().into_iter().map(|v| -> Result<OutPoint> {
      let mut cursor = Cursor::new(v.as_ref().clone());
      Ok(consensus_decode::<bitcoin::blockdata::transaction::OutPoint>(&mut cursor)?)
    }).collect::<Result<Vec<OutPoint>>>()?;
    let mut result: WalletResponse = WalletResponse::new();
    for outpoint in outpoints {
        let outpoint_bytes = outpoint_to_bytes(&outpoint)?;
        let _address = tables::OUTPOINT_SPENDABLE_BY.select(&outpoint_bytes).get();
        if address.len() == _address.len() {
            let balance_sheet: BalanceSheet = BalanceSheet::load(
                &atomic.derive(
                    &tables::RUNES.OUTPOINT_TO_RUNES.select(&outpoint_bytes)
                )
            );
            let decoded_output: Output = Output::parse_from_bytes(
              &tables::OUTPOINT_TO_OUTPUT.select(&outpoint_bytes).get().as_ref()
            )?;

            let final_outpoint: OutpointResponse = OutpointResponse {
                balances: MessageField::some(balance_sheet_to_proto(&BalanceSheet::default())),
                outpoint: MessageField::some(core_outpoint_to_proto(&outpoint)),
                output: MessageField::some(decoded_output),
                height: 0,
                txindex: 0,
                special_fields: SpecialFields::new()
            };
            result.outpoints.push(final_outpoint);
        }
    }
    Ok(result)
}
