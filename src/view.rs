use protorune_support::balance_sheet::{ BalanceSheet, ProtoruneRuneId };
use crate::proto::protorune::{
    BalanceSheet as ProtoBalanceSheet,
    BalanceSheetItem,
    Outpoint,
    OutpointResponse,
    Output,
    Rune,
    RuneId,
    //RunesByHeightRequest,
    RunesResponse,
    WalletResponse,
};
use protorune_support::utils::{ consensus_decode };
use crate::{ balance_sheet::{ load_sheet }, proto, tables };
use anyhow::{ anyhow, Result };
use bitcoin;
//use bitcoin::consensus::Decodable;
use bitcoin::hashes::Hash;
use bitcoin::OutPoint;
//use hex;
use metashrew_support::byte_view::ByteView;
//use metashrew::utils::{ consume_exact, consume_sized_int };
use metashrew::index_pointer::{ AtomicPointer, KeyValuePointer };
use protobuf::{ Message, MessageField, SpecialFields };
use std::collections::HashMap;
use std::io::Cursor;
use std::collections::HashSet;

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
                        v.rune.runeId.txindex as u128
                    );
                    (id, u128::from_bytes(v.balance))
                })
            ),
        }
    }
}

impl From<BalanceSheet> for ProtoBalanceSheet {
    fn from(balance_sheet: BalanceSheet) -> ProtoBalanceSheet {
        ProtoBalanceSheet {
            entries: balance_sheet.balances
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
                        symbol: 'Z' as char as u32,
                    }),
                    balance: (&v.to_le_bytes()).to_vec(),
                })
                .collect::<Vec<BalanceSheetItem>>(),
            special_fields: SpecialFields::new(),
        }
    }
}
pub fn protorune_outpoint_to_outpoint_response(
    outpoint: &OutPoint,
    protocol_id: u128
) -> Result<OutpointResponse> {
    let outpoint_bytes = outpoint_to_bytes(outpoint)?;
    let balance_sheet: BalanceSheet = load_sheet(
        &tables::RuneTable::for_protocol(protocol_id).OUTPOINT_TO_RUNES.select(&outpoint_bytes)
    );

    let mut height: u128 = tables::RuneTable
        ::for_protocol(protocol_id)
        .OUTPOINT_TO_HEIGHT.select(&outpoint_bytes)
        .get_value::<u64>()
        .into();
    let mut txindex: u128 = tables::RuneTable
        ::for_protocol(protocol_id)
        .HEIGHT_TO_TRANSACTION_IDS.select_value::<u64>(height as u64)
        .get_list()
        .into_iter()
        .position(|v| v.as_ref().to_vec() == outpoint.txid.as_byte_array().to_vec())
        .ok_or("")
        .map_err(|_| anyhow!("txid not indexed in table"))? as u128;

    if let Some((rune_id, _)) = balance_sheet.clone().balances.iter().next() {
        height = rune_id.block;
        txindex = rune_id.tx;
    }
    let decoded_output: Output = Output::parse_from_bytes(
        &tables::OUTPOINT_TO_OUTPUT.select(&outpoint_bytes).get().as_ref()
    )?;
    Ok(OutpointResponse {
        balances: MessageField::some(balance_sheet.into()),
        outpoint: MessageField::some(core_outpoint_to_proto(&outpoint)),
        output: MessageField::some(decoded_output),
        height: height as u32,
        txindex: txindex as u32,
        special_fields: SpecialFields::new(),
        address: vec![],
    })
}

pub fn outpoint_to_outpoint_response(outpoint: &OutPoint) -> Result<OutpointResponse> {
    let outpoint_bytes = outpoint_to_bytes(outpoint)?;
    let balance_sheet: BalanceSheet = load_sheet(
        &tables::RUNES.OUTPOINT_TO_RUNES.select(&outpoint_bytes)
    );
    let mut height: u128 = tables::RUNES.OUTPOINT_TO_HEIGHT
        .select(&outpoint_bytes)
        .get_value::<u64>()
        .into();
    let mut txindex: u128 = tables::RUNES.HEIGHT_TO_TRANSACTION_IDS
        .select_value::<u64>(height as u64)
        .get_list()
        .into_iter()
        .position(|v| v.as_ref().to_vec() == outpoint.txid.as_byte_array().to_vec())
        .ok_or("")
        .map_err(|_| anyhow!("txid not indexed in table"))? as u128;

    if let Some((rune_id, _)) = balance_sheet.clone().balances.iter().next() {
        height = rune_id.block;
        txindex = rune_id.tx;
    }
    let decoded_output: Output = Output::parse_from_bytes(
        &tables::OUTPOINT_TO_OUTPUT.select(&outpoint_bytes).get().as_ref()
    )?;
    Ok(OutpointResponse {
        balances: MessageField::some(balance_sheet.into()),
        outpoint: MessageField::some(core_outpoint_to_proto(&outpoint)),
        output: MessageField::some(decoded_output),
        height: height as u32,
        txindex: txindex as u32,
        special_fields: SpecialFields::new(),
        address: vec![],
    })
}

pub fn runes_by_address(input: &Vec<u8>) -> Result<WalletResponse> {
    let mut result: WalletResponse = WalletResponse::new();
    if let Some(req) = proto::protorune::WalletRequest::parse_from_bytes(input).ok() {
        result.outpoints = tables::OUTPOINTS_FOR_ADDRESS
            .select(&req.wallet)
            .get_list()
            .into_iter()
            .map(
                |v| -> Result<OutPoint> {
                    let mut cursor = Cursor::new(v.as_ref().clone());
                    Ok(consensus_decode::<bitcoin::blockdata::transaction::OutPoint>(&mut cursor)?)
                }
            )
            .collect::<Result<Vec<OutPoint>>>()?
            .into_iter()
            .filter_map(
                |v| -> Option<Result<OutpointResponse>> {
                    let outpoint_bytes = match outpoint_to_bytes(&v) {
                        Ok(v) => v,
                        Err(e) => {
                            return Some(Err(e));
                        }
                    };
                    let _address = tables::OUTPOINT_SPENDABLE_BY.select(&outpoint_bytes).get();
                    if req.wallet.len() == _address.len() {
                        Some(outpoint_to_outpoint_response(&v))
                    } else {
                        None
                    }
                }
            )
            .collect::<Result<Vec<OutpointResponse>>>()?;
    }
    Ok(result)
}

pub fn protorunes_by_address(input: &Vec<u8>) -> Result<WalletResponse> {
    let mut result: WalletResponse = WalletResponse::new();
    if let Some(req) = proto::protorune::ProtorunesWalletRequest::parse_from_bytes(input).ok() {
        result.outpoints = tables::OUTPOINTS_FOR_ADDRESS
            .select(&req.wallet)
            .get_list()
            .into_iter()
            .map(
                |v| -> Result<OutPoint> {
                    let mut cursor = Cursor::new(v.as_ref().clone());
                    Ok(consensus_decode::<bitcoin::blockdata::transaction::OutPoint>(&mut cursor)?)
                }
            )
            .collect::<Result<Vec<OutPoint>>>()?
            .into_iter()
            .filter_map(
                |v| -> Option<Result<OutpointResponse>> {
                    let outpoint_bytes = match outpoint_to_bytes(&v) {
                        Ok(v) => v,
                        Err(e) => {
                            return Some(Err(e));
                        }
                    };
                    let _address = tables::OUTPOINT_SPENDABLE_BY.select(&outpoint_bytes).get();
                    if req.wallet.len() == _address.len() {
                        Some(
                            protorune_outpoint_to_outpoint_response(
                                &v,
                                u128::from_bytes(req.clone().protocol_tag)
                            )
                        )
                    } else {
                        None
                    }
                }
            )
            .collect::<Result<Vec<OutpointResponse>>>()?;
    }
    Ok(result)
}

pub fn protorune_holders(input: &Vec<u8>) -> Result<WalletResponse> {
    let mut result: WalletResponse = WalletResponse::new();

    if let Some(req) = proto::protorune::ProtoruneHoldersRequest::parse_from_bytes(input).ok() {
        let rune_id = &req.id;
        let rune_id_str = rune_id.to_string();
        let protocol_tag = u128::from_bytes(req.clone().protocol_tag);

        // Select appropriate table
        let table = if protocol_tag != 0 {
            tables::RuneTable::for_protocol(protocol_tag)
        } else {
            tables::RUNES.clone()
        };

        // The structure we're using is:
        // Key: rune_id string
        // Value: list of outpoint bytes

        // Get all outpoints for this rune ID
        let outpoints_list = table.RUNE_OUTPOINT_MAPPING
            .select(&rune_id_str.into_bytes())
            .get_list();

        // Process each outpoint
        result.outpoints = outpoints_list
            .into_iter()
            .filter_map(|outpoint_bytes_arc| {
                // Check if this outpoint is active by checking its balance
                let mut cursor = Cursor::new(outpoint_bytes_arc.as_ref().clone());

                match consensus_decode::<OutPoint>(&mut cursor) {
                    Ok(outpoint) => {
                        // Convert outpoint to bytes format for lookup
                        match outpoint_to_bytes(&outpoint) {
                            Ok(outpoint_bytes) => {
                                // Get the balance sheet for this outpoint
                                let sheet = load_sheet(
                                    &AtomicPointer::default().derive(
                                        &table.OUTPOINT_TO_RUNES.select(&outpoint_bytes)
                                    )
                                );

                                // Check if this outpoint has a positive balance of the rune
                                let protorune_id = ProtoruneRuneId {
                                    block: u128::from(rune_id.height),
                                    tx: u128::from(rune_id.txindex),
                                };
                                let balance = sheet.get(&protorune_id);

                                // Only include outpoints with positive balances
                                if balance > 0 {
                                    // Get the address that can spend this outpoint
                                    let address = tables::OUTPOINT_SPENDABLE_BY
                                        .select(&outpoint_bytes)
                                        .get();

                                    // Create response
                                    let response = protorune_outpoint_to_outpoint_response(
                                        &outpoint,
                                        protocol_tag
                                    );

                                    if let Ok(mut resp) = response {
                                        // Check if address Arc is empty or not
                                        if !address.is_empty() {
                                            if
                                                let Ok(address_str) = String::from_utf8(
                                                    address.as_ref().clone()
                                                )
                                            {
                                                resp.address = address_str.into_bytes();
                                            }
                                        }
                                        Some(Ok(resp))
                                    } else {
                                        Some(response)
                                    }
                                } else {
                                    None // Skip outpoints with zero balance
                                }
                            }
                            Err(e) => Some(Err(e)),
                        }
                    }
                    Err(_) => None,
                }
            })
            .collect::<Result<Vec<OutpointResponse>>>()?;
    }

    Ok(result)
}

pub fn runes_by_height(input: &Vec<u8>) -> Result<RunesResponse> {
    let mut result: RunesResponse = RunesResponse::new();
    if let Some(req) = proto::protorune::RunesByHeightRequest::parse_from_bytes(input).ok() {
        for rune in tables::HEIGHT_TO_RUNES.select_value(req.height).get_list().into_iter() {
            let mut _rune: Rune = Rune::new();
            _rune.name = rune.clone().to_vec();
            _rune.runeId = MessageField::from_option(
                RuneId::parse_from_bytes(&tables::RUNES.ETCHING_TO_RUNE_ID.select(&rune).get()).ok()
            );
            _rune.spacers = tables::RUNES.SPACERS.select(&rune).get_value::<u32>();

            _rune.symbol = tables::RUNES.SYMBOL.select(&rune).get_value::<u32>();
            _rune.divisibility = tables::RUNES.DIVISIBILITY.select(&rune).get_value::<u8>() as u32;
            result.runes.push(_rune);
        }
    }
    Ok(result)
}
