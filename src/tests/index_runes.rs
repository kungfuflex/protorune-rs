#[cfg(test)]
mod tests {
    use crate::{ constants, view };
    use crate::message::MessageContext;
    use crate::tests::helpers;
    use crate::Protorune;
    use bitcoin::consensus::serialize;
    use bitcoin::hashes::Hash;
    use bitcoin::{ blockdata::block::Block, Address };
    use bitcoin::{ OutPoint, Txid };
    use metashrew::{
        get_cache,
        clear,
        utils::{ format_key },
        flush,
        index_pointer::{ IndexPointer, KeyValuePointer },
        println,
        stdio::stdout,
    };
    use ruint::uint;
    use std::fmt::Write;
    use std::str::FromStr;
    use std::sync::Arc;
    use wasm_bindgen_test::*;
    use hex;

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
        constants::OUTPOINTS_FOR_ADDRESS
            .keyword("bc1qcr8te4kr609gcawutmrza0j4xv80jy8z306fyu")
            .set(Arc::new(Vec::new()));
        let test_val = IndexPointer::from_keyword("/blockhash/byheight/")
            .select_value(840000 as u32)
            .get();
        let hex_str = display_vec_as_hex((*test_val).clone()); // Dereference and clone the Vec<u8>
        println!("{}", hex_str);
        assert_eq!(hex_str, expected_block_hash);
    }

    #[wasm_bindgen_test]
    fn spendable_by_address() {
        clear();
        let test_block = helpers::create_block_with_tx();
        let _ = Protorune::index_block::<MyMessageContext>(test_block.clone(), 840001);
        constants::OUTPOINTS_FOR_ADDRESS
            .keyword("bc1qcr8te4kr609gcawutmrza0j4xv80jy8z306fyu")
            .set(Arc::new(Vec::new()));
        let outpoint: OutPoint = OutPoint {
            txid: Txid::from_str(
                "a440cb400062f14cff5f76fbbd3881c426820171180c67c103a36d12c89fbd32"
            ).unwrap(),
            vout: 0,
        };
        let test_val = constants::OUTPOINT_SPENDABLE_BY.select(&serialize(&outpoint)).get();
        let addr_str = display_vec_as_hex(test_val.to_vec());
        let _addr_str: String = display_vec_as_hex(
            "bc1qcr8te4kr609gcawutmrza0j4xv80jy8z306fyu".to_string().into_bytes()
        );

        let view_test = Protorune::outpoints_by_address(
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
        let test_block = helpers::create_block_with_tx();
        constants::OUTPOINTS_FOR_ADDRESS
            .keyword("bc1qcr8te4kr609gcawutmrza0j4xv80jy8z306fyu")
            .set(Arc::new(Vec::new()));
        let _ = Protorune::index_block::<MyMessageContext>(test_block.clone(), 840001);
        let outpoint: OutPoint = OutPoint {
            txid: Txid::from_str(
                "a440cb400062f14cff5f76fbbd3881c426820171180c67c103a36d12c89fbd32"
            ).unwrap(),
            vout: 0,
        };
        let test_val = constants::OUTPOINTS_FOR_ADDRESS
            .keyword("bc1qcr8te4kr609gcawutmrza0j4xv80jy8z306fyu")
            .get_list();
        let list_str: String = display_list_as_hex(test_val);

        let test_outpoint: Vec<u8> = serialize(&outpoint);
        let outpoint_hex: String = display_vec_as_hex(test_outpoint);

        assert_eq!(list_str, outpoint_hex);
    }
}
