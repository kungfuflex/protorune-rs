use anyhow::Result;
use bitcoin::consensus::{
    deserialize_partial,
    encode::{Decodable, Encodable},
};
use metashrew_support::utils::{is_empty, remaining_slice};
use ordinals::varint;
use std::io::BufRead;
pub fn consensus_encode<T: Encodable>(v: &T) -> Result<Vec<u8>> {
    let mut result = Vec::<u8>::new();
    <T as Encodable>::consensus_encode::<Vec<u8>>(v, &mut result)?;
    Ok(result)
}

pub fn consensus_decode<T: Decodable>(cursor: &mut std::io::Cursor<Vec<u8>>) -> Result<T> {
    let slice = &cursor.get_ref()[cursor.position() as usize..cursor.get_ref().len() as usize];
    let deserialized: (T, usize) = deserialize_partial(slice)?;
    cursor.consume(deserialized.1);
    Ok(deserialized.0)
}

pub fn decode_varint_list(cursor: &mut std::io::Cursor<Vec<u8>>) -> Result<Vec<u128>> {
    let mut result: Vec<u128> = vec![];
    while !is_empty(cursor) {
        let (n, sz) = varint::decode(remaining_slice(cursor))?;
        cursor.consume(sz);
        result.push(n);
    }
    Ok(result)
}

/// returns the values in a LEB encoded stream
pub fn encode_varint_list(values: &Vec<u128>) -> Vec<u8> {
    let mut result = Vec::<u8>::new();
    for value in values {
        varint::encode_to_vec(*value, &mut result);
    }
    result
}

pub fn field_to_name(data: &u128) -> String {
    let mut v = data + 1; // Increment by 1
    let mut result = String::new();
    let twenty_six: u128 = 26;

    while v > 0 {
        let mut y = (v % twenty_six) as u32;
        if y == 0 {
            y = 26;
        }

        // Convert number to character (A-Z, where A is 65 in ASCII)
        result.insert(0, char::from_u32(64 + y).unwrap());

        v -= 1; // Decrement v by 1
        v /= twenty_six; // Divide v by 26 for next iteration
    }

    result
}
