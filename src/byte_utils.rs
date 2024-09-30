use metashrew_rs::byte_view::{shrink_back, ByteView};

pub trait ByteUtils {
    pub fn to_aligned_bytes(v: Self) -> Vec<u8>;
    pub fn snap_to_15_bytes(v: Self) -> Vec<u8>;
}

impl ByteUtils for u128 {
    fn to_aligned_bytes(v: Self) -> Vec<u8> {
        let mut ar = ByteView::to_bytes(v);
        let mut end = 0;
        for (i, v) in ar.into_iter().enumerate() {
            if v != 0 {
                break;
            }
            end = i
        }

        ar.drain(0..end);
        ar
    }
    fn snap_to_15_bytes(v: Self) -> Vec<u8> {
        let mut ar = ByteView::to_bytes(v);
        ar.drain(0..1);
        ar
    }
}
