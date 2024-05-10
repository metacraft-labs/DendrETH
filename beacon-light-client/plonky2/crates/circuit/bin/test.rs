use circuit::array::Array;

pub fn main() {
    let asd = Array([1, 2, 3]);
    println!("{}", asd.len());

    for i in asd.iter() {
        println!("asd[i] = {i}");
    }
}
