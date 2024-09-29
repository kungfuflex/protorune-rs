pub mod helpers;
mod tests {
    use crate::helpers;
    use wasm_bindgen_test::*;
    use bitcoin::blockdata::block::Block;
    use protorune_rs::message::MessageContext;
    use protorune_rs::{Protorune};
    use ruint::uint;

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
        assert!(Protorune::index_block::<MyMessageContext>(helpers::create_block_with_coinbase(840000), 840000).is_ok());
    }

    #[wasm_bindgen_test]
    fn hello_world() {
        assert_eq!(protorune_rs::hello_world(), "hello_world")
    }

}
