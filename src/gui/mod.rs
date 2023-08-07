use std::vec::Vec;

fn normalize_buffer(buffer: Vec<i32>) -> Vec<Vec<i32>> {
    let mut out_v: Vec<Vec<i32>> = Vec::new();

    for x in 0..(buffer.len() / 3) {
        out_v.append(&mut vec![buffer[x..(x + 3)].to_vec()]);
    }

    out_v
}
