#[cfg(test)]
mod tests {
    use crate::balance_sheet::{BalanceSheet, ProtoruneRuneId};
    use crate::message::MessageContext;
    use crate::tests::helpers;
    use crate::tests::helpers::{display_list_as_hex, display_vec_as_hex};
    use crate::utils::consensus_encode;
    use crate::view::View;
    use crate::Protorune;
    use crate::{constants, message::MessageContextParcel, tables, view};
    use bitcoin::consensus::serialize;
    use bitcoin::hashes::Hash;
    use bitcoin::{blockdata::block::Block, Address};
    use bitcoin::{OutPoint, Txid};
    use hex;
    use metashrew::{
        clear, flush, get_cache,
        index_pointer::{IndexPointer, KeyValuePointer},
        println,
        stdio::stdout,
        utils::format_key,
    };
    use ordinals::Rune;
    use ruint::uint;
    use std::fmt::Write;
    use std::str::FromStr;
    use std::sync::Arc;
    use wasm_bindgen_test::*;

    struct MyMessageContext(());

    impl MessageContext for MyMessageContext {
        fn handle(_parcel: Box<MessageContextParcel>) -> bool {
            false
        }
        fn protocol_tag() -> u128 {
            100
        }
    }

    #[wasm_bindgen_test]
    fn height_blockhash() {
        clear();
        let test_block = helpers::create_block_with_coinbase_tx(840000);
        let expected_block_hash =
            display_vec_as_hex(test_block.block_hash().as_byte_array().to_vec());
        let _ = Protorune::index_block::<MyMessageContext>(test_block.clone(), 840000);
        let test_height_to_blockhash = tables::RUNES
            .HEIGHT_TO_BLOCKHASH
            .select_value(840000 as u64)
            .get();
        let test_blockhash_to_height = tables::RUNES
            .BLOCKHASH_TO_HEIGHT
            .select(&test_block.block_hash().as_byte_array().to_vec())
            .get_value::<u64>();
        assert_eq!(
            hex::encode(test_height_to_blockhash.as_ref()),
            expected_block_hash
        );
        assert_eq!(test_blockhash_to_height, 840000);
    }

    #[wasm_bindgen_test]
    fn spendable_by_address() {
        clear();
        let test_block = helpers::create_block_with_sample_tx();
        let _ = Protorune::index_block::<MyMessageContext>(test_block.clone(), 840001);
        tables::OUTPOINTS_FOR_ADDRESS
            .keyword("bc1qcr8te4kr609gcawutmrza0j4xv80jy8z306fyu")
            .set(Arc::new(Vec::new()));
        let outpoint: OutPoint = OutPoint {
            txid: Txid::from_str(
                "a440cb400062f14cff5f76fbbd3881c426820171180c67c103a36d12c89fbd32",
            )
            .unwrap(),
            vout: 0,
        };
        let test_val = tables::OUTPOINT_SPENDABLE_BY
            .select(&serialize(&outpoint))
            .get();
        let addr_str = display_vec_as_hex(test_val.to_vec());
        let _addr_str: String = display_vec_as_hex(
            "bc1qcr8te4kr609gcawutmrza0j4xv80jy8z306fyu"
                .to_string()
                .into_bytes(),
        );

        let view_test = View::outpoints_by_address(
            "bc1qcr8te4kr609gcawutmrza0j4xv80jy8z306fyu"
                .to_string()
                .into_bytes(),
        );
        let mut outpoint_vec: Vec<String> = Vec::new();
        outpoint_vec
            .push("a440cb400062f14cff5f76fbbd3881c426820171180c67c103a36d12c89fbd32:0".to_string());
        let matching_view_test = view::AddressOutpoints {
            outpoints: outpoint_vec,
        };
        assert_eq!(
            view_test,
            serde_json::to_string_pretty(&matching_view_test).unwrap()
        );
        assert_eq!(_addr_str, addr_str);
    }

    #[wasm_bindgen_test]
    fn outpoints_by_address() {
        clear();
        let test_block = helpers::create_block_with_sample_tx();
        let _ = Protorune::index_block::<MyMessageContext>(test_block.clone(), 840001);
        let outpoint: OutPoint = OutPoint {
            txid: Txid::from_str(
                "a440cb400062f14cff5f76fbbd3881c426820171180c67c103a36d12c89fbd32",
            )
            .unwrap(),
            vout: 0,
        };
        let test_val = tables::OUTPOINTS_FOR_ADDRESS
            .keyword("bc1qcr8te4kr609gcawutmrza0j4xv80jy8z306fyu")
            .get_list();
        let list_str: String = display_list_as_hex(test_val);

        let test_outpoint: Vec<u8> = serialize(&outpoint);
        let outpoint_hex: String = display_vec_as_hex(test_outpoint);

        assert_eq!(list_str, outpoint_hex);
    }

    #[wasm_bindgen_test]
    fn index_runestone() {
        clear();
        let (test_block, config) = helpers::create_block_with_rune_tx();
        tables::OUTPOINTS_FOR_ADDRESS
            .keyword(&config.address1)
            .set(Arc::new(Vec::new()));
        let _ =
            Protorune::index_block::<MyMessageContext>(test_block.clone(), config.rune_etch_height);
        let rune_id = Protorune::build_rune_id(config.rune_etch_height, config.rune_etch_vout);
        let test_val = tables::RUNES.RUNE_ID_TO_ETCHING.select(&rune_id).get();
        let cache_hex: String = display_vec_as_hex(test_val.to_vec());
        let rune = Rune::from_str(&config.rune_name)
            .unwrap()
            .0
            .to_string()
            .into_bytes();
        let rune_hex: String = display_vec_as_hex(rune);
        assert_eq!(rune_hex, cache_hex);
    }

    #[wasm_bindgen_test]
    fn correct_balance_sheet() {
        clear();
        let (test_block, config) = helpers::create_block_with_rune_tx();
        let _ =
            Protorune::index_block::<MyMessageContext>(test_block.clone(), config.rune_etch_height);
        let outpoint: OutPoint = OutPoint {
            txid: test_block.txdata[0].txid(),
            vout: 0,
        };
        let protorune_id = ProtoruneRuneId {
            block: config.rune_etch_height as u128,
            tx: config.rune_etch_vout as u128,
        };
        let sheet = BalanceSheet::load(
            &tables::RUNES
                .OUTPOINT_TO_RUNES
                .select(&consensus_encode(&outpoint).unwrap()),
        );
        let stored_balance = sheet.get(&protorune_id);
        println!("stored balance: {}", stored_balance);
        assert_eq!(1000 as u128, stored_balance);
    }

    #[wasm_bindgen_test]
    fn correct_balance_sheet_with_transfers() {
        clear();
        let (test_block, config) = helpers::create_block_with_rune_transfer();
        let _ =
            Protorune::index_block::<MyMessageContext>(test_block.clone(), config.rune_etch_height);
        let outpoint_address2: OutPoint = OutPoint {
            txid: test_block.txdata[1].txid(),
            vout: 0,
        };
        let outpoint_address1: OutPoint = OutPoint {
            txid: test_block.txdata[1].txid(),
            vout: 1,
        };
        let protorune_id = ProtoruneRuneId {
            block: config.rune_etch_height as u128,
            tx: config.rune_etch_vout as u128,
        };
        let sheet1 = BalanceSheet::load(
            &tables::RUNES
                .OUTPOINT_TO_RUNES
                .select(&consensus_encode(&outpoint_address1).unwrap()),
        );
        let stored_balance_address1 = sheet1.get(&protorune_id);
        assert_eq!(800 as u128, stored_balance_address1);

        let sheet2 = BalanceSheet::load(
            &tables::RUNES
                .OUTPOINT_TO_RUNES
                .select(&consensus_encode(&outpoint_address2).unwrap()),
        );
        let stored_balance_address2 = sheet2.get(&protorune_id);
        assert_eq!(200 as u128, stored_balance_address2);
    }
}
