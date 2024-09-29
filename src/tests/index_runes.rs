#[cfg(test)]
mod tests {
    use crate::message::MessageContext;
    use crate::tests::helpers;
    use crate::Protorune;
    use bitcoin::blockdata::block::Block;
    use bitcoin::hashes::Hash;
    use metashrew_rs::{flush, index_pointer::IndexPointer, println, stdio::stdout};
    use ruint::uint;
    use std::fmt::Write;
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

    #[wasm_bindgen_test]
    fn protorune_creation() {
        let test_block = helpers::create_block_with_coinbase(840000);
        let expected_block_hash =
            display_vec_as_hex(test_block.block_hash().as_byte_array().to_vec());
        Protorune::index_block::<MyMessageContext>(test_block, 840000);
        let test_val = IndexPointer::from_keyword("/blockhash/byheight/")
            .select_value(840000 as u32)
            .get();
        let hex_str = display_vec_as_hex((*test_val).clone()); // Dereference and clone the Vec<u8>
        println!("{}", hex_str);
        assert_eq!(hex_str, expected_block_hash);
    }

    #[wasm_bindgen_test]
    fn hello_world() {
        assert_eq!("hello_world", "hello_world")
    }
    #[wasm_bindgen_test]
    fn test_println() {
        println!("test println");
    }
}
