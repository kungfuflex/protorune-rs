use bitcoin::address::NetworkChecked;
use bitcoin::blockdata::block::{Block, Header, Version};
use bitcoin::blockdata::script::ScriptBuf;
use bitcoin::blockdata::transaction::{Transaction, TxIn, TxOut};
use bitcoin::hashes::Hash;
use bitcoin::string::FromHexStr;
use bitcoin::{Address, Amount, BlockHash, OutPoint, Sequence, Witness};
use byteorder::{ByteOrder, LittleEndian};
use core::str::FromStr;
use ordinals::{Etching, Rune, Runestone};

pub fn serialize_u32_little_endian(value: u32) -> Vec<u8> {
    let mut buf = vec![0u8; 4]; // Create a buffer of 4 bytes
    LittleEndian::write_u32(&mut buf, value); // Write the value in little-endian order
    buf
}

pub fn create_coinbase_transaction(height: u32) -> Transaction {
    // Create the script for the coinbase transaction
    let script_pubkey = Address::from_str("bc1qcr8te4kr609gcawutmrza0j4xv80jy8z306fyu")
        .unwrap()
        .require_network(bitcoin::Network::Bitcoin)
        .unwrap()
        .script_pubkey();
    // Create a coinbase transaction input
    let coinbase_input = TxIn {
        previous_output: Default::default(),
        script_sig: ScriptBuf::new(),
        sequence: Sequence::MAX, // sequence for coinbase
        witness: Witness::new(),
    };

    // Create the coinbase transaction output
    let coinbase_output = TxOut {
        value: 50_000_000, // 50 BTC in satoshis
        script_pubkey,
    };

    let locktime = bitcoin::absolute::LockTime::from_height(height).unwrap();

    // Create the coinbase transaction
    Transaction {
        version: 2,
        lock_time: locktime,
        input: vec![coinbase_input],
        output: vec![coinbase_output],
    }
}

pub fn create_block_with_coinbase(height: u32) -> Block {
    // Create the coinbase transaction
    let coinbase_tx = create_coinbase_transaction(height);

    // Define block header fields
    let version = Version::from_consensus(2);
    let previous_blockhash =
        BlockHash::from_str("00000000000000000005c3b409b4f17f9b3a97ed46d1a63d3f660d24168b2b3e")
            .unwrap();

    // let merkle_root_hash = bitcoin::merkle_tree::calculate_root(&[coinbase_tx.clone()]);
    let merkle_root = bitcoin::hash_types::TxMerkleNode::from_str(
        "4e07408562b4b5a9c0555f0671e0d2b6c5764c1d2a5e97c1d7f36f7c91e4c77a",
    )
    .unwrap();
    let time = 1231006505; // Example timestamp (January 3, 2009)
    let bits = bitcoin::CompactTarget::from_hex_str("0x1234").unwrap(); // Example bits (difficulty)
    let nonce = 2083236893; // Example nonce

    // Create the block header
    let header = Header {
        version,
        prev_blockhash: previous_blockhash,
        merkle_root,
        time,
        bits,
        nonce,
    };

    Block {
        header: header,
        txdata: vec![coinbase_tx],
    }
}

pub fn serialize_block(block: &Block) -> [u8; 32] {
    block.block_hash().to_raw_hash().to_byte_array()
}

pub fn create_test_transaction() -> Transaction {
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

    let address_str = "bc1qcr8te4kr609gcawutmrza0j4xv80jy8z306fyu";

    let address: Address<NetworkChecked> = Address::from_str(&address_str)
        .unwrap()
        .require_network(bitcoin::Network::Bitcoin)
        .unwrap();

    let script_pubkey = address.script_pubkey();

    let txout = TxOut {
        value: Amount::from_sat(100_000_000).to_sat(),
        script_pubkey,
    };

    Transaction {
        version: 1,
        lock_time: bitcoin::absolute::LockTime::ZERO, // no locktime
        input: vec![txin],
        output: vec![txout],
    }
}

pub fn create_rune_transaction() -> Transaction {
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

    let address_str = "bc1qcr8te4kr609gcawutmrza0j4xv80jy8z306fyu";

    let address: Address<NetworkChecked> = Address::from_str(&address_str)
        .unwrap()
        .require_network(bitcoin::Network::Bitcoin)
        .unwrap();

    let script_pubkey = address.script_pubkey();

    let txout = TxOut {
        value: Amount::from_sat(100_000_000).to_sat(),
        script_pubkey,
    };

    let runestone: ScriptBuf = (Runestone {
        etching: Some(Etching {
            divisibility: Some(2),
            premine: Some(1000),
            rune: Some(Rune::from_str("TESTER").unwrap()),
            spacers: Some(0),
            symbol: Some(char::from_str("Z").unwrap()),
            turbo: true,
            terms: None,
        }),
        pointer: Some(0),
        edicts: Vec::new(),
        mint: None,
        proto: None,
    })
    .encipher();

    let op_return = TxOut {
        value: Amount::from_sat(0).to_sat(),
        script_pubkey: runestone,
    };

    Transaction {
        version: 1,
        lock_time: bitcoin::absolute::LockTime::ZERO,
        input: vec![txin],
        output: vec![txout, op_return],
    }
}

pub fn create_block_with_tx(rune: bool) -> Block {
    let tx: Transaction;
    if rune {
        tx = create_rune_transaction();
    } else {
        tx = create_test_transaction();
    }

    // Define block header fields
    let version = Version::from_consensus(1);
    let previous_blockhash =
        BlockHash::from_str("00000000000000000005c3b409b4f17f9b3a97ed46d1a63d3f660d24168b2b3e")
            .unwrap();

    // let merkle_root_hash = bitcoin::merkle_tree::calculate_root(&[coinbase_tx.clone()]);
    let merkle_root = bitcoin::hash_types::TxMerkleNode::from_str(
        "4e07408562b4b5a9c0555f0671e0d2b6c5764c1d2a5e97c1d7f36f7c91e4c77a",
    )
    .unwrap();
    let time = 1231006505; // Example timestamp (January 3, 2009)
    let bits = bitcoin::CompactTarget::from_hex_str("0x1234").unwrap(); // Example bits (difficulty)
    let nonce = 2083236893; // Example nonce

    // Create the block header
    let header = Header {
        version,
        prev_blockhash: previous_blockhash,
        merkle_root,
        time,
        bits,
        nonce,
    };

    // Create the block with the coinbase transaction
    Block {
        header,
        txdata: vec![tx],
    }
}
