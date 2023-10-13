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
