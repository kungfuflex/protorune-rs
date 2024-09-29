use std::u128;

pub trait MessageContext {
    fn handle() -> bool;
    fn protocol_tag() -> u128;
}
