#[cfg(test)]
mod tests {
    use crate::balance_sheet::{BalanceSheet, ProtoruneRuneId};
    use crate::message::{MessageContext, MessageContextParcel};
    use crate::protostone::{Protostone, Protostones};
    use crate::rune_transfer::RuneTransfer;
    use crate::tests::helpers::{self, get_address, ADDRESS1};
    use crate::Protorune;
    use anyhow::Result;
    use bitcoin::Transaction;
    use bitcoin::{
        address::NetworkChecked, Address, Amount, OutPoint, ScriptBuf, Sequence, TxIn, TxOut,
        Witness,
    };
    use metashrew::clear;
    use ordinals::{Etching, Rune, Runestone};
    use std::str::FromStr;
    use wasm_bindgen_test::wasm_bindgen_test;

    struct TestMessageContext(());

    impl MessageContext for TestMessageContext {
        fn protocol_tag() -> u128 {
            1
        }
        fn handle(parcel: &MessageContextParcel) -> Result<(Vec<RuneTransfer>, BalanceSheet)> {
            let mut new_runtime_balances = parcel.runtime_balances.clone();
            <BalanceSheet as TryFrom<Vec<RuneTransfer>>>::try_from(parcel.runes.clone())?
                .pipe(&mut new_runtime_balances);
            Ok((vec![], *new_runtime_balances))
        }
    }

    #[wasm_bindgen_test]
    fn protomessage_test() {
        clear();
        let mut test_block = helpers::create_block_with_coinbase_tx(840000);
        let previous_output = OutPoint {
            txid: bitcoin::Txid::from_str(
                "0000000000000000000000000000000000000000000000000000000000000000",
            )
            .unwrap(),
            vout: 0,
        };
        let input_script = ScriptBuf::new();

        // Create a transaction input
        let txin = TxIn {
            previous_output,
            script_sig: input_script,
            sequence: Sequence::MAX,
            witness: Witness::new(),
        };

        let address: Address<NetworkChecked> = get_address(&ADDRESS1);

        let script_pubkey = address.script_pubkey();

        // tx vout 0 will hold all 1000 of the runes
        let txout = TxOut {
            value: Amount::from_sat(100_000_000).to_sat(),
            script_pubkey,
        };

        let runestone: ScriptBuf = (Runestone {
            etching: Some(Etching {
                divisibility: Some(2),
                premine: Some(1000),
                rune: Some(Rune::from_str("TESTTESTTEST").unwrap()),
                spacers: Some(0),
                symbol: Some(char::from_str("A").unwrap()),
                turbo: true,
                terms: None,
            }),
            pointer: Some(1),
            edicts: Vec::new(),
            mint: None,
            protocol: match vec![
                Protostone {
                    burn: Some(0u32),
                    edicts: vec![],
                    pointer: Some(3),
                    refund: None,
                    from: None,
                    protocol_tag: 1,
                    message: vec![],
                },
                Protostone {
                    message: vec![1u8],
                    pointer: Some(0),
                    refund: Some(0),
                    edicts: vec![],
                    from: None,
                    burn: None,
                    protocol_tag: 1,
                },
            ]
            .encipher()
            {
                Ok(v) => Some(v),
                Err(_) => None,
            },
        })
        .encipher();

        let op_return = TxOut {
            value: Amount::from_sat(0).to_sat(),
            script_pubkey: runestone,
        };

        test_block.txdata.push(Transaction {
            version: 1,
            lock_time: bitcoin::absolute::LockTime::ZERO,
            input: vec![txin],
            output: vec![txout, op_return],
        });
        Protorune::index_block::<TestMessageContext>(test_block.clone(), 840000);
    }
}
