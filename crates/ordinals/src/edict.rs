use super::*;

#[derive(Default, Serialize, Deserialize, Debug, PartialEq, Copy, Clone, Eq)]
pub struct Edict {
    pub id: RuneId,
    pub amount: u128,
    pub output: u32,
}

impl Edict {
    pub fn from_integers(
        num_outputs: u32,
        id: RuneId,
        amount: u128,
        output: u128,
        check_outputs: bool,
    ) -> Option<Self> {
        let Ok(output) = u32::try_from(output) else {
            return None;
        };

        // note that this allows `output == tx.output.len()`, which means to divide
        // amount between all non-OP_RETURN outputs
        if check_outputs && output > num_outputs {
            return None;
        }

        Some(Self { id, amount, output })
    }
}
