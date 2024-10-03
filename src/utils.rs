use anyhow::{Result};
use bitcoin::consensus::encode::{Encodable};
pub fn consensus_encode<T: Encodable>(v: &T) -> Result<Vec<u8>> {
  let mut result = Vec::<u8>::new();
  <T as Encodable>::consensus_encode::<Vec<u8>>(v, &mut result)?;
  Ok(result)
}
