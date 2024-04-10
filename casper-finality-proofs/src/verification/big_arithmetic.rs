pub fn big_less_than(a: &[u32], b: &[u32]) -> bool {
    assert_eq!(a.len(), b.len());
    for i in (0..a.len()).rev() {
        if a[i] < b[i] {
            return true;
        } else if b[i] < a[i] {
            return false;
        }
    }
    false
}

pub fn big_add(a: &[u32], b: &[u32]) -> Vec<u32> {
    assert_eq!(a.len(), b.len());
    let mut c: Vec<u32> = Vec::with_capacity(a.len()+1);
    let mut carry: u32 = 0;
    for (a_i, b_i) in a.iter().zip(b.iter()) {
        let c_i = (*a_i as u64) + (*b_i as u64) + (carry as u64);
        c.push(c_i as u32);
        carry = (c_i >> 32) as u32;
    }
    c.push(carry as u32);
    c
}

pub fn big_sub(a: &[u32], b: &[u32]) -> (Vec<u32>, u32) {
    // assume a>b
    assert_eq!(a.len(), b.len());
    let mut c: Vec<u32> = Vec::with_capacity(a.len());
    let mut carry: u32 = 0;
    for (a_i, b_i) in a.iter().zip(b.iter()) {
        let b_plus_carry: u64 = (*b_i as u64) + (carry as u64);
        if *a_i as u64 >= b_plus_carry {
            c.push(a_i - (b_plus_carry as u32));
            carry = 0;
        } else {
            c.push(((1u64<<32) + (*a_i as u64) - b_plus_carry) as u32);
            carry = 1;
        }
    }
    (c, carry)
}

// a * b
pub fn big_multiply(a: &[u32], b: &[u32]) -> Vec<u32> {
    assert_eq!(a.len(), b.len());
    let mut c: Vec<u32> = Vec::with_capacity(a.len()+1);
    let mut carry = 0;
    for (a_i, b_i) in a.iter().zip(b.iter()) {
        let c_i = (*a_i as u64) * (*b_i as u64) + (carry as u64);
        c.push(c_i as u32);
        carry = (c_i >> 32) as u32;
    }
    c.push(carry as u32);
    c
}

#[cfg(test)]
mod tests {
    use crate::big_arithmetic::{big_less_than, big_sub};

    use super::big_add;

    #[test]
    fn test_big_add() {
        let a = vec![1<<31, ((1u64<<32)-1) as u32, 1];
        let b = vec![1<<31, 1, 4];
        let ans = vec![0, 1, 6, 0];
        let big_add_ans = big_add(&a, &b);
        assert_eq!(ans, big_add_ans);
    }

    #[test]
    fn test_less_than() {
        let a = vec![0,1,2];
        let b = vec![2,3,1];
        assert_eq!(big_less_than(&a, &b), false);
        assert_eq!(big_less_than(&b, &a), true);
        assert_eq!(big_less_than(&b, &b), false);
    }

    #[test]
    fn test_big_sub() {
        let a = vec![1<<31, 3, 1];
        let b = vec![1<<31, 1, 1];
        let ans = vec![0, ((1u64<<32) - 2) as u32, ((1u64<<32) - 1) as u32];
        let (sub_ans, carry) = big_sub(&b, &a);
        assert_eq!(ans, sub_ans);
        assert_eq!(carry, 1);

        let ans = vec![0, 2, 0];
        let (sub_ans, carry) = big_sub(&a, &b);
        assert_eq!(ans, sub_ans);
        assert_eq!(carry, 0);
    }
}
