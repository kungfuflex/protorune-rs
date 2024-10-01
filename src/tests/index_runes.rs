#[cfg(test)]
mod tests {
    use crate::message::MessageContext;
    use crate::tests::helpers;
    use crate::Protorune;
    use crate::constants;
    use bitcoin::consensus::serialize;
    use bitcoin::{ OutPoint, Txid };
    use bitcoin::{ blockdata::block::Block, Address };
    use bitcoin::hashes::Hash;
    use metashrew_rs::{ flush, index_pointer::{IndexPointer, KeyValuePointer}, println, stdio::stdout };
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

        // Iterate through each Arc<Vec<u8>> in the Vec
        for arc_data in data {
            // Dereference the Arc and iterate through the inner Vec<u8>
            for byte in arc_data.iter() {
                write!(&mut hex_string, "{:02x}", byte).expect("Unable to write");
            }
        }

        hex_string
    }

    #[wasm_bindgen_test]
    fn protorune_creation() {
        let test_block = helpers::create_block_with_coinbase(840000);
        let expected_block_hash = display_vec_as_hex(
            test_block.block_hash().as_byte_array().to_vec()
        );
        let _ = Protorune::index_block::<MyMessageContext>(test_block, 840000);
        let test_val = IndexPointer::from_keyword("/blockhash/byheight/")
            .select_value(840000 as u32)
            .get();
        let hex_str = display_vec_as_hex((*test_val).clone()); // Dereference and clone the Vec<u8>
        println!("{}", hex_str);
        assert_eq!(hex_str, expected_block_hash);
    }

    #[wasm_bindgen_test]
    fn outpoints_by_address() {
        let test_block = helpers::create_block_with_tx();
        let _ = Protorune::index_block::<MyMessageContext>(test_block.clone(), 840001);
        let test_val = constants::OUTPOINTS_FOR_ADDRESS
            .select(&"bc1qcr8te4kr609gcawutmrza0j4xv80jy8z306fyu".to_string().into_bytes())
            .get_list();

        let hex_str = display_list_as_hex(test_val.clone());
        let outpoint: OutPoint = OutPoint {
            txid: Txid::from_str(
                "a440cb400062f14cff5f76fbbd3881c426820171180c67c103a36d12c89fbd32"
            ).unwrap(),
            vout: 0,
        };

        let test_outpoint: Vec<u8> = serialize(&outpoint);
        let outpoint_hex: String = display_vec_as_hex(test_outpoint);

        assert_eq!(hex_str, outpoint_hex);
    }

}
