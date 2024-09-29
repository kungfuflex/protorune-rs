use bitcoin::blockdata::block::{Block, Header, Version};
use bitcoin::blockdata::script::ScriptBuf;
use bitcoin::blockdata::transaction::{Transaction, TxIn, TxOut};
use bitcoin::hashes::Hash;
use bitcoin::string::FromHexStr;
use bitcoin::{BlockHash, Sequence, Witness};
use byteorder::{ByteOrder, LittleEndian};
use core::str::FromStr;

pub fn serialize_u32_little_endian(value: u32) -> Vec<u8> {
    let mut buf = vec![0u8; 4]; // Create a buffer of 4 bytes
    LittleEndian::write_u32(&mut buf, value); // Write the value in little-endian order
    buf
}

pub fn create_coinbase_transaction(height: u32) -> Transaction {
    // Create the script for the coinbase transaction
    // let script_pubkey = ScriptBuf::new_p2wpkh(&Address::from_str("bc1q...").unwrap().payload).unwrap();
    let script_pubkey = ScriptBuf::new();
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
        script_pubkey: script_pubkey,
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

    // Create the block with the coinbase transaction
    Block {
        header: header,
        txdata: vec![coinbase_tx],
    }
}

pub fn serialize_block(block: &Block) -> [u8; 32] {
    block.block_hash().to_raw_hash().to_byte_array()
}
