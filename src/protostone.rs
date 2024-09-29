use crate::protoburn::Protoburn;
use ordinals::{Edict, Runestone};

pub struct Protostone {
    burns: Option<Vec<Protoburn>>,
    messages: Option<u32>,
    edicts: Option<Vec<Edict>>,
    refund: Option<u32>,
}

impl Protostone {
    pub fn from_runestone(runestone: Runestone) -> anyhow::Result<()> {
        let mut fields = runestone.fields;

        Ok(())
    }
}
