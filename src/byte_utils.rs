
pub trait ByteUtils {
    fn to_aligned_bytes(self) -> Vec<u8>;
    fn snap_to_15_bytes(self) -> Vec<u8>;
    fn to_u32(self) -> u32;
}

impl ByteUtils for u128 {
    fn to_aligned_bytes(self) -> Vec<u8> {
        let mut ar: Vec<u8> = (self.to_le_bytes()).try_into().unwrap();
        let mut end = 0;
        for (_i, v) in ar.iter().enumerate() {
            if *v != 0 {
                break;
            }
            end = end + 1;
        }

        ar.drain(0..end);
        ar
    }
    fn snap_to_15_bytes(self) -> Vec<u8> {
        let mut ar: Vec<u8> = (self.to_le_bytes()).try_into().unwrap();
        ar.drain(0..1);
        ar
    }

    fn to_u32(self) -> u32 {
      self as u32
    }
}
