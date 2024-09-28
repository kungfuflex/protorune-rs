use ruint::Uint;

pub trait MessageContext {
    fn handle() -> bool;
    fn protocol_tag() -> Uint<128, 2>;
}
