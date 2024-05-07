use std::str::FromStr;

use num_bigint::BigUint;

fn main() {
    let a = BigUint::from_str("1015072001812290770271495995578254894147382487313523610684315265448920391983183057185266070149383515536696015791412").unwrap();

    println!("A: {}", a);

    println!("A limbs: {:?}", a.to_u32_digits());
}
