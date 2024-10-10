use crate::balance_sheet::{BalanceSheet, ProtoruneRuneId};
use crate::proto::protorune::{
    BalanceSheet as ProtoBalanceSheet, BalanceSheetItem, Outpoint, OutpointResponse, Output, Rune,
    RuneId, WalletResponse,
};
use crate::tables;
use crate::utils::{consensus_decode, consensus_encode};
use anyhow::{anyhow, Result};
use bitcoin;
use bitcoin::consensus::Decodable;
use bitcoin::hashes::{sha256d, Hash};
use bitcoin::OutPoint;
use hex;
use metashrew::byte_view::ByteView;
use metashrew::utils::{consume_exact, consume_sized_int};
use metashrew::{
    index_pointer::{AtomicPointer, KeyValuePointer},
    println,
    stdio::stdout,
};
use protobuf::{Message, MessageField, SpecialFields};
use std::collections::HashMap;
use std::fmt::Write;
use std::io::Cursor;

pub fn outpoint_to_bytes(outpoint: &OutPoint) -> Result<Vec<u8>> {
    let mut result = Vec::<u8>::with_capacity(0x24);
    result.extend(&outpoint.txid.as_byte_array().to_vec());
    result.extend(&outpoint.vout.to_le_bytes());
    Ok(result)
}

pub fn core_outpoint_to_proto(outpoint: &OutPoint) -> Outpoint {
    Outpoint {
        txid: outpoint.txid.as_byte_array().to_vec().clone(),
        vout: outpoint.vout,
        special_fields: SpecialFields::new(),
    }
}

impl From<ProtoBalanceSheet> for BalanceSheet {
    fn from(balance_sheet: ProtoBalanceSheet) -> BalanceSheet {
        BalanceSheet {
            balances: HashMap::<ProtoruneRuneId, u128>::from_iter(
                balance_sheet.entries.into_iter().map(|v| {
                    let id = ProtoruneRuneId::new(
                        v.rune.runeId.height as u128,
                        v.rune.runeId.txindex as u128,
                    );
                    (id, u128::from_bytes(v.balance))
                }),
            ),
        }
    }
}

impl From<BalanceSheet> for ProtoBalanceSheet {
    fn from(balance_sheet: BalanceSheet) -> ProtoBalanceSheet {
        ProtoBalanceSheet {
            entries: balance_sheet
                .balances
                .clone()
                .iter()
                .map(|(k, v)| BalanceSheetItem {
                    special_fields: SpecialFields::new(),
                    rune: MessageField::some(Rune {
                        special_fields: SpecialFields::new(),
                        runeId: MessageField::some(RuneId {
                            special_fields: SpecialFields::new(),
                            height: k.block as u32,
                            txindex: k.tx as u32,
                        }),
                        name: "name".as_bytes().to_vec(),
                        divisibility: 1,
                        spacers: 1,
                        symbol: 1,
                    }),
                    balance: (&v.to_le_bytes()).to_vec(),
                })
                .collect::<Vec<BalanceSheetItem>>(),
            special_fields: SpecialFields::new(),
        }
    }
}

pub fn outpoint_to_outpoint_response(outpoint: &OutPoint) -> Result<OutpointResponse> {
    let outpoint_bytes = outpoint_to_bytes(outpoint)?;
    let balance_sheet: BalanceSheet =
        BalanceSheet::load(&tables::RUNES.OUTPOINT_TO_RUNES.select(&outpoint_bytes));
    let mut height: u128 = tables::RUNES
        .OUTPOINT_TO_HEIGHT
        .select(&outpoint_bytes)
        .get_value::<u64>()
        .into();
    let mut txindex: u128 = tables::RUNES
        .HEIGHT_TO_TRANSACTION_IDS
        .select_value::<u64>(height as u64)
        .get_list()
        .into_iter()
        .position(|v| {
            v.as_ref().to_vec() == outpoint.txid.as_byte_array().to_vec()
        })
        .ok_or("")
        .map_err(|_| anyhow!("txid not indexed in table"))? as u128;

    if let Some((rune_id, _)) = balance_sheet.clone().balances.iter().next() {
        height = rune_id.block;
        txindex = rune_id.tx;
    }
    let decoded_output: Output = Output::parse_from_bytes(
        &tables::OUTPOINT_TO_OUTPUT
            .select(&outpoint_bytes)
            .get()
            .as_ref(),
    )?;
    Ok(OutpointResponse {
        balances: MessageField::some(balance_sheet.into()),
        outpoint: MessageField::some(core_outpoint_to_proto(&outpoint)),
        output: MessageField::some(decoded_output),
        height: height as u32,
        txindex: txindex as u32,
        special_fields: SpecialFields::new(),
    })
}

pub fn runes_by_address(address: &Vec<u8>) -> Result<WalletResponse> {
    let mut result: WalletResponse = WalletResponse::new();
    result.outpoints = tables::OUTPOINTS_FOR_ADDRESS
        .select(address)
        .get_list()
        .into_iter()
        .map(|v| -> Result<OutPoint> {
            let mut cursor = Cursor::new(v.as_ref().clone());
            Ok(consensus_decode::<bitcoin::blockdata::transaction::OutPoint>(&mut cursor)?)
        })
        .collect::<Result<Vec<OutPoint>>>()?
        .into_iter()
        .filter_map(|v| -> Option<Result<OutpointResponse>> {
            let outpoint_bytes = match outpoint_to_bytes(&v) {
                Ok(v) => v,
                Err(e) => {
                    return Some(Err(e));
                }
            };
            let _address = tables::OUTPOINT_SPENDABLE_BY.select(&outpoint_bytes).get();
            if address.len() == _address.len() {
                Some(outpoint_to_outpoint_response(&v))
            } else {
                None
            }
        })
        .collect::<Result<Vec<OutpointResponse>>>()?;
    Ok(result)
}
