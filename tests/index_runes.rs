use protorune_rs::message::MessageContext;
use protorune_rs::Protorune;
use ruint::uint;

struct MyMessageContext;

impl MessageContext for MyMessageContext {
    fn handle() -> bool {
        false
    }
    fn protocol_tag() -> ruint::Uint<128, 2> {
        uint!(100_U128)
    }
}

#[test]
fn protorune_creation() {
    assert!(Protorune::index_block::<MyMessageContext>().is_ok());
}
