use metashrew::byte_view::{shrink_back, ByteView};

pub trait ByteUtils {
    fn to_aligned_bytes(v: Self) -> Vec<u8>;
    fn snap_to_15_bytes(v: Self) -> Vec<u8>;
    fn to_u32(v: Self) -> u32;
}

impl ByteUtils for u128 {
    fn to_aligned_bytes(v: Self) -> Vec<u8> {
        let mut ar = ByteView::to_bytes(v);
        let mut end = 0;
        for (i, v) in ar.iter().enumerate() {
            if *v != 0 {
                break;
            }
            end = end + 1;
        }

        ar.drain(0..end);
        ar
    }
    fn snap_to_15_bytes(v: Self) -> Vec<u8> {
        let mut ar = ByteView::to_bytes(v);
        ar.drain(0..1);
        ar
    }

    fn to_u32(v: Self) -> u32 {
        let ar = ByteView::to_bytes(v);
        ByteView::from_bytes(ar[0..4].to_vec())
    }
}
