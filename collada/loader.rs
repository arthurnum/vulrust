use std::fs::File;
use std::io::Read;
use std::mem::transmute;

fn main() {
    let mut file = File::open("cube.data").unwrap();

    let mut data: Vec<f32> = Vec::new();
    let mut double_buffer: [u8; 4] = [0; 4];

    let mut read = true;

    while read {
        match file.read(&mut double_buffer) {
            Ok(size) => {
                if size < 1 { read = false }
            }
            Err(err) => read = false
        }

        if read {
            unsafe {
                data.push(transmute::<[u8; 4], f32>(double_buffer));
            }
        }
    }

    println!("{:?}", data);
}
