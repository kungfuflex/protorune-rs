mod view {
    use crate::constants;
    use metashrew::index_pointer::KeyValuePointer;
    use std::sync::Arc;

    pub fn outpoints_by_address(address: Vec<u8>) -> Vec<Arc<Vec<u8>>> {
        let outpoints = constants::OUTPOINTS_FOR_ADDRESS.select(&address).get_list();
        let mut ret: Vec<Arc<Vec<u8>>> = Vec::new();
        for outpoint in outpoints {
            let _address = constants::OUTPOINT_SPENDABLE_BY.select(&outpoint).get();
            if address.len() == _address.len() {
                ret.push(outpoint);
            }
        }
        return ret;
    }
}

//   findOutpointsForAddress(address: ArrayBuffer): Array<ArrayBuffer> {
//     const addressPtr = OUTPOINTS_FOR_ADDRESS.select(address);
//     const keys = new Array<ArrayBuffer>(0);

//     for (let i: u32 = 0; i < addressPtr.length(); i++) {
//       const item = addressPtr.selectIndex(i).get();
//       const _address = OUTPOINT_SPENDABLE_BY.select(item).get();
//       if (
//         memory.compare(
//           changetype<usize>(address),
//           changetype<usize>(_address),
//           address.byteLength,
//         ) == 0
//       ) {
//         keys.push(item);
//       }
//     }

//     return keys;
