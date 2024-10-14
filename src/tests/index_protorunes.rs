#[cfg(test)]
mod tests {
    use crate::balance_sheet::{BalanceSheet, ProtoruneRuneId};
    use crate::message::{MessageContext, MessageContextParcel};
    use crate::protostone::{Protostone, Protostones};
    use crate::rune_transfer::RuneTransfer;
    use crate::tests::helpers::{self, get_address, ADDRESS1};
    use crate::utils::consensus_encode;
    use crate::{tables, Protorune};
    use anyhow::Result;
    use bitcoin::{
        address::NetworkChecked, Address, Amount, OutPoint, ScriptBuf, Sequence, TxIn, TxOut,
        Witness,
    };
    use bitcoin::{block, Transaction};
    use metashrew::clear;
    use metashrew::index_pointer::KeyValuePointer;
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
            // transfer protorunes to the pointer
            Ok((vec![], *new_runtime_balances))
        }
    }

    /// In one runestone, etches a rune, then protoburns it
    #[wasm_bindgen_test]
    fn protomessage_test() {
        clear();
        let block_height = 840000;
        let protocol_id = 1;
        let mut test_block = helpers::create_block_with_coinbase_tx(block_height);

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
            pointer: Some(1), // points to the OP_RETURN, so therefore targets the protoburn
            edicts: Vec::new(),
            mint: None,
            protocol: match vec![
                Protostone {
                    // protoburn and give protorunes to output 3 (virtual output, the protomessage)
                    burn: Some(protocol_id),
                    edicts: vec![],
                    pointer: Some(3),
                    refund: None,
                    from: None,
                    protocol_tag: 13, // this value must be 13 if protoburn
                    message: vec![],
                },
                Protostone {
                    // protomessage which should transfer protorunes to the pointer
                    message: vec![1u8],
                    pointer: Some(0),
                    refund: Some(0),
                    edicts: vec![],
                    from: None,
                    burn: None,
                    protocol_tag: protocol_id as u128,
                },
            ]
            .encipher()
            {
                Ok(v) => Some(v),
                Err(_) => None,
            },
        })
        .encipher();

        // op return is at output 1
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
        assert!(Protorune::index_block::<TestMessageContext>(
            test_block.clone(),
            block_height as u64
        )
        .is_ok());

        // tx 0 is coinbase, tx 1 is runestone
        let outpoint_address: OutPoint = OutPoint {
            txid: test_block.txdata[1].txid(),
            vout: 0,
        };
        // check runes balance
        let sheet = BalanceSheet::load(
            &tables::RUNES
                .OUTPOINT_TO_RUNES
                .select(&consensus_encode(&outpoint_address).unwrap()),
        );

        let protorunes_sheet = BalanceSheet::load(
            &tables::RuneTable::for_protocol(protocol_id.into())
                .OUTPOINT_TO_RUNES
                .select(&consensus_encode(&outpoint_address).unwrap()),
        );

        let protorune_id = ProtoruneRuneId {
            block: block_height as u128,
            tx: 1,
        };
        let stored_balance_address = sheet.get(&protorune_id);
        assert_eq!(stored_balance_address, 0);

        let stored_protorune_balance = protorunes_sheet.get(&protorune_id);
        assert_eq!(stored_protorune_balance, 1000);
    }
}
