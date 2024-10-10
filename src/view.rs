use crate::balance_sheet::BalanceSheet;
use crate::proto::protorune::{
    OutpointResponse,
    WalletResponse,
    Output,
    Outpoint,
    BalanceSheet as ProtoBalanceSheet,
};
use crate::tables;
use crate::utils::consensus_encode;
use bitcoin::consensus::Decodable;
use bitcoin::hashes::{ sha256d, Hash };
use bitcoin::OutPoint;
use metashrew::index_pointer::{ AtomicPointer, KeyValuePointer };
use protobuf::{ Message, MessageField };

pub fn runes_by_address(address: Vec<u8>) -> WalletResponse {
    let atomic = AtomicPointer::default();
    let outpoints = tables::OUTPOINTS_FOR_ADDRESS.select(&address).get_list();
    let mut ret: WalletResponse = WalletResponse::new();
    for outpoint in outpoints {
        let _address = tables::OUTPOINT_SPENDABLE_BY.select(&outpoint).get();
        if address.len() == _address.len() {
            let balance_sheet: BalanceSheet = BalanceSheet::load(
                &atomic.derive(
                    &tables::RUNES.OUTPOINT_TO_RUNES.select(&consensus_encode(&outpoint).unwrap())
                )
            );
            let height: u32 = 0;
            let txindex: u32 = 0;
            let decoded_output: Output = Output::parse_from_bytes(
                &tables::OUTPOINT_TO_OUTPUT.select(&*&outpoint).get()
            ).unwrap();

            let final_outpoint: OutpointResponse = OutpointResponse {
                balances: MessageField::some(
                    ProtoBalanceSheet::parse_from_bytes(
                        &serde_json::to_vec(&balance_sheet).unwrap()
                    ).unwrap()
                ),
                outpoint: MessageField::some(Outpoint::parse_from_bytes(&outpoint).unwrap()),
                output: MessageField::some(decoded_output.clone()),
                height,
                txindex,
                special_fields: decoded_output.special_fields,
            };
            ret.outpoints.push(final_outpoint);
        }
    }
    return ret;
}
