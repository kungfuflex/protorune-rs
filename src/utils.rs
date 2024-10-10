use anyhow::Result;
use metashrew::utils::{ is_empty, remaining_slice, consume_to_end };
use ordinals::varint;
use bitcoin::consensus::{ deserialize, encode::{Encodable, Decodable } };
use std::io::BufRead;
use std::io::Cursor;
pub fn consensus_encode<T: Encodable>(v: &T) -> Result<Vec<u8>> {
    let mut result = Vec::<u8>::new();
    <T as Encodable>::consensus_encode::<Vec<u8>>(v, &mut result)?;
    Ok(result)
}

pub fn consensus_decode<T: Decodable>(cursor: &mut std::io::Cursor<Vec<u8>>) -> Result<T> {
  Ok(deserialize(&consume_to_end(cursor)?)?)
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
