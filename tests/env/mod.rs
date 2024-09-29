use wasm_bindgen::prelude::*;

static mut _INPUT: Option<Vec<u8>> = None;
#[wasm_bindgen(module = "env")]
pub fn __host_len() -> i32 {
    unsafe {
        match _INPUT.as_ref() {
            Some(v) => v.len() as i32,
            None => 0,
        }
    }
}

#[wasm_bindgen(module = "env")]
pub fn __load_input(ptr: i32) -> () {
    unsafe {
        match _INPUT.as_ref() {
            Some(v) => {
              (&mut std::slice::from_raw_parts_mut(ptr as usize as *mut u8, v.len()))
                .clone_from_slice(&*v)
                },
            None => (),
        }
    }
}
