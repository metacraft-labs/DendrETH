/// Asserts that two values are equal.
/// When the assertion fails, the macro will return an Err with a message containing values,
/// file, line and column where the assertion failed.
#[macro_export]
macro_rules! assert_equal {
    ($left:expr, $right:expr $(,)?) => {
        match (&$left, &$right) {
            (left_val, right_val) => {
                if !(*left_val == *right_val) {
                    Err(anyhow::anyhow!(format!(
                        "{}\n  - at {}:{}:{}",
                        format!("{:?} != {:?}", left_val, right_val),
                        file!(),
                        line!(),
                        column!()
                    )))?;
                }
            }
        }
    };
}

/// Converts primitive types such as H128, H256, H384, H512, U256, U512 to string.
/// The default to_string implementation is not in valid hex format.
#[macro_export]
macro_rules! to_string {
    ($value:expr) => {
        format!("{:?}", $value)
    };
}
