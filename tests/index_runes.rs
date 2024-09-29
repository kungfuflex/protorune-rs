mod helpers;

mod tests {
    use crate::helpers;

    use super::helpers::create_block_with_coinbase;
    use bitcoin::blockdata::block::Block;
    use protorune_rs::message::MessageContext;
    use protorune_rs::{BlockInput, Protorune};
    use ruint::uint;

    struct BlockInputData {
        height: u32,
        block: Block,
    }

    impl BlockInputData {
        fn serialize(&self) -> Vec<u8> {
            let mut out = helpers::serialize_u32_little_endian(self.height);
            let block_bytes = helpers::serialize_block(&self.block);
            out.extend_from_slice(&block_bytes);
            return out;
        }
    }
    struct MyMessageContext;

    impl MessageContext for MyMessageContext {
        fn handle() -> bool {
            false
        }
        fn protocol_tag() -> ruint::Uint<128, 2> {
            uint!(100_U128)
        }
    }

    struct MockBlockInput;

    impl BlockInput for MockBlockInput {
        fn get_input() -> Vec<u8> {
            let height = 840000;
            let block = create_block_with_coinbase(height);
            let block_data = BlockInputData { height, block };
            return block_data.serialize();
        }
    }

    #[test]
    fn protorune_creation() {
        assert!(Protorune::<MockBlockInput>::index_block::<MyMessageContext>().is_ok());
    }

    #[test]
    fn hello_world() {
        assert_eq!(protorune_rs::hello_world(), "hello_world")
    }
}
