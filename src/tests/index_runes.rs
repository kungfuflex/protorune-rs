#[cfg(test)]
mod tests {
    use crate::balance_sheet::{ BalanceSheet, ProtoruneRuneId };
    use crate::message::MessageContext;
    use crate::tests::helpers;
    use crate::utils::consensus_encode;
    use crate::view::View;
    use crate::Protorune;
    use crate::{ constants, tables, view };
    use bitcoin::consensus::serialize;
    use bitcoin::hashes::Hash;
    use bitcoin::{ blockdata::block::Block, Address };
    use bitcoin::{ OutPoint, Txid };
    use hex;
    use metashrew::{
        clear,
        flush,
        get_cache,
        index_pointer::{ IndexPointer, KeyValuePointer },
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
        fn handle() -> bool {
            false
        }
        fn protocol_tag() -> u128 {
            100
        }
    }

    fn display_vec_as_hex(data: Vec<u8>) -> String {
        let mut hex_string = String::new();
        for byte in data {
            write!(&mut hex_string, "{:02x}", byte).expect("Unable to write");
        }
        hex_string
    }

    fn display_list_as_hex(data: Vec<Arc<Vec<u8>>>) -> String {
        let mut hex_string = String::new();

        for arc_data in data {
            for byte in arc_data.to_vec().iter() {
                write!(&mut hex_string, "{:02x}", byte).expect("Unable to write");
            }
        }

        hex_string
    }

    #[wasm_bindgen_test]
    fn protorune_creation() {
        clear();
        let test_block = helpers::create_block_with_coinbase(840000);
        let expected_block_hash = display_vec_as_hex(
            test_block.block_hash().as_byte_array().to_vec()
        );
        let _ = Protorune::index_block::<MyMessageContext>(test_block, 840000);
        tables::OUTPOINTS_FOR_ADDRESS
            .keyword("bc1qcr8te4kr609gcawutmrza0j4xv80jy8z306fyu")
            .set(Arc::new(Vec::new()));
        let test_val = tables::RUNES.HEIGHT_TO_BLOCKHASH
            .select_value(840000 as u64)
            .get();
        let hex_str = hex::encode(test_val.as_ref());
        println!("{}", &hex_str);
        assert_eq!(hex_str, expected_block_hash);
    }

    #[wasm_bindgen_test]
    fn spendable_by_address() {
        clear();
        let test_block = helpers::create_block_with_tx(false);
        let _ = Protorune::index_block::<MyMessageContext>(test_block.clone(), 840001);
        tables::OUTPOINTS_FOR_ADDRESS
            .keyword("bc1qcr8te4kr609gcawutmrza0j4xv80jy8z306fyu")
            .set(Arc::new(Vec::new()));
        let outpoint: OutPoint = OutPoint {
            txid: Txid::from_str(
                "a440cb400062f14cff5f76fbbd3881c426820171180c67c103a36d12c89fbd32"
            ).unwrap(),
            vout: 0,
        };
        let test_val = tables::OUTPOINT_SPENDABLE_BY.select(&serialize(&outpoint)).get();
        let addr_str = display_vec_as_hex(test_val.to_vec());
        let _addr_str: String = display_vec_as_hex(
            "bc1qcr8te4kr609gcawutmrza0j4xv80jy8z306fyu".to_string().into_bytes()
        );

        let view_test = View::outpoints_by_address(
            "bc1qcr8te4kr609gcawutmrza0j4xv80jy8z306fyu".to_string().into_bytes()
        );
        let mut outpoint_vec: Vec<String> = Vec::new();
        outpoint_vec.push(
            "a440cb400062f14cff5f76fbbd3881c426820171180c67c103a36d12c89fbd32:0".to_string()
        );
        let matching_view_test = view::AddressOutpoints {
            outpoints: outpoint_vec,
        };
        assert_eq!(view_test, serde_json::to_string_pretty(&matching_view_test).unwrap());
        assert_eq!(_addr_str, addr_str);
    }

    #[wasm_bindgen_test]
    fn outpoints_by_address() {
        clear();
        let test_block = helpers::create_block_with_tx(false);
        let _ = Protorune::index_block::<MyMessageContext>(test_block.clone(), 840001);
        let outpoint: OutPoint = OutPoint {
            txid: Txid::from_str(
                "a440cb400062f14cff5f76fbbd3881c426820171180c67c103a36d12c89fbd32"
            ).unwrap(),
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
        let test_block = helpers::create_block_with_tx(true);
        tables::OUTPOINTS_FOR_ADDRESS
            .keyword("bc1qcr8te4kr609gcawutmrza0j4xv80jy8z306fyu")
            .set(Arc::new(Vec::new()));
        let _ = Protorune::index_block::<MyMessageContext>(test_block.clone(), 840001);
        let rune_id = Protorune::build_rune_id(840001, 0);
        let test_val = tables::RUNES.RUNE_ID_TO_ETCHING.select(&rune_id).get();
        let cache_hex: String = display_vec_as_hex(test_val.to_vec());
        let rune = Rune::from_str("TESTER").unwrap().0.to_string().into_bytes();
        let rune_hex: String = display_vec_as_hex(rune);
        assert_eq!(rune_hex, cache_hex);
    }

    #[wasm_bindgen_test]
    fn correct_balance_sheet() {
        clear();
        let test_block = helpers::create_block_with_tx(true);
        let _ = Protorune::index_block::<MyMessageContext>(test_block.clone(), 840000);
        let outpoint: OutPoint = OutPoint {
            txid: Txid::from_str(
                "9ce5f5651eb9042179e8dca3d51aa74eb2973637f8b1a0c878920dab1b0724fc"
            ).unwrap(),
            vout: 0,
        };
        let protorune_id = ProtoruneRuneId {
            block: 840000 as u128,
            tx: 0 as u128,
        };
        let sheet = BalanceSheet::load(
            &tables::RUNES.OUTPOINT_TO_RUNES.select(&consensus_encode(&outpoint).unwrap())
        );

        let stored_balance = sheet.get(&protorune_id);
        assert_eq!(1000 as u128, stored_balance);
    }
}
