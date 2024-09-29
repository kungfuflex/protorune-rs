#[cfg(test)]
mod tests {
    use crate::tests::helpers;
    use bitcoin::blockdata::block::Block;
    use crate::message::MessageContext;
    use crate::{
      Protorune
    };
    use ruint::uint;
    use wasm_bindgen_test::*;

    struct MyMessageContext(());

    impl MessageContext for MyMessageContext {
        fn handle() -> bool {
            false
        }
        fn protocol_tag() -> ruint::Uint<128, 2> {
            uint!(100_U128)
        }
    }

    #[wasm_bindgen_test]
    fn protorune_creation() {
        assert!(Protorune::index_block::<MyMessageContext>(
            helpers::create_block_with_coinbase(840000),
            840000
        )
        .is_ok());
    }

    #[wasm_bindgen_test]
    fn hello_world() {
        assert_eq!("hello_world", "hello_world")
    }
}
