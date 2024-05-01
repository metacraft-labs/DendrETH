// BLS Native

use std::ops::{Add, Div, Mul, Neg, Sub};

use std::{str::FromStr, vec};

use num_bigint::{BigInt, BigUint, Sign, ToBigInt};

use super::big_arithmetic::{self, big_add, big_less_than};

pub fn modulus() -> BigUint {
    BigUint::from_str("4002409555221667393417789825735904156556882819939007885332058136124031650490837864442687629129015664037894272559787").unwrap()
}

pub fn modulus_digits() -> Vec<u32> {
    modulus().to_u32_digits()
}

pub fn get_bls_12_381_parameter() -> BigUint {
    BigUint::from_str("15132376222941642752").unwrap()
}

pub fn get_negate(y: &[u32; 12]) -> [u32; 12] {
    let y_bu = BigUint::new(y.to_vec());
    let neg = modulus() - y_bu;
    get_u32_vec_from_literal(neg)
}

pub fn get_g2_invert(z1: &[u32; 12], z2: &[u32; 12]) -> [[u32; 12]; 2] {
    let fp2 = Fp2([Fp(z1.clone()), Fp(z2.clone())]);
    [fp2.invert().0[0].0, fp2.invert().0[1].0]
}

pub fn get_u32_carries(x: &[u32; 12], y: &[u32; 12]) -> [u32; 12] {
    let mut carries = [0u32; 12];
    let mut prev_carry = 0;
    for i in 0..12 {
        if i != 0 {
            prev_carry = carries[i - 1];
        }
        let z = (x[i] as u64) + (y[i] as u64) + (prev_carry as u64);
        println!(
            "i-{:?}--x:: {:?}, y:: {:?}, z:: {:?}, carry:: {:?}",
            i,
            x[i],
            y[i],
            prev_carry,
            (z >> 32) as u32
        );
        if i != 11 {
            carries[i] = (z >> 32) as u32
        }
    }
    carries[11] = 0;
    carries
}

pub fn multiply_by_slice(x: &[u32; 12], y: u32) -> ([u32; 13], [u32; 12]) {
    let mut res: [u32; 13] = [0u32; 13];
    let mut carries: [u32; 12] = [0u32; 12];
    let mut prev_carry = 0;
    for i in 0..12 {
        let temp = (x[i] as u64 * y as u64) + prev_carry as u64;
        let temp_res = temp as u32;
        let new_carry = (temp >> 32) as u32;
        prev_carry = new_carry;
        res[i] = temp_res;
        carries[i] = prev_carry;
    }
    res[12] = prev_carry;
    (res, carries)
}

pub fn add_u32_slices(x: &[u32; 24], y: &[u32; 24]) -> ([u32; 24], [u32; 24]) {
    let mut prev_carry = 0u32;
    let mut res = [0u32; 24];
    let mut carries = [0u32; 24];
    for i in 0..24 {
        let s = x[i] as u64 + y[i] as u64 + prev_carry as u64;
        let sum = s as u32;
        let carry = (s >> 32) as u32;
        prev_carry = carry;
        res[i] = sum;
        carries[i] = carry;
    }
    (res, carries)
}

pub fn add_u32_slices_12(x: &[u32; 12], y: &[u32; 12]) -> ([u32; 12], [u32; 12]) {
    let mut prev_carry = 0u32;
    let mut res = [0u32; 12];
    let mut carries = [0u32; 12];
    for i in 0..12 {
        let s = x[i] as u64 + y[i] as u64 + prev_carry as u64;
        let sum = s as u32;
        let carry = (s >> 32) as u32;
        prev_carry = carry;
        res[i] = sum;
        carries[i] = carry;
    }
    (res, carries)
}

// assume x > y
pub fn sub_u32_slices(x: &[u32; 24], y: &[u32; 24]) -> ([u32; 24], [u32; 24]) {
    let mut prev_borrow = 0u32;
    let mut res = [0u32; 24];
    let mut borrows = [0u32; 24];
    for i in 0..24 {
        if x[i] >= y[i] + prev_borrow {
            res[i] = x[i] - y[i] - prev_borrow;
            borrows[i] = 0;
            prev_borrow = 0;
        } else {
            res[i] = ((1u64 << 32) + x[i] as u64 - y[i] as u64 - prev_borrow as u64) as u32;
            borrows[i] = 1;
            prev_borrow = 1;
        }
    }
    (res, borrows)
}

// assume x > y
pub fn sub_u32_slices_12(x: &[u32; 12], y: &[u32; 12]) -> ([u32; 12], [u32; 12]) {
    let mut prev_borrow = 0u32;
    let mut res = [0u32; 12];
    let mut borrows = [0u32; 12];
    for i in 0..12 {
        if x[i] >= y[i] + prev_borrow {
            res[i] = x[i] - y[i] - prev_borrow;
            borrows[i] = 0;
            prev_borrow = 0;
        } else {
            res[i] = ((1u64 << 32) + x[i] as u64 - y[i] as u64 - prev_borrow as u64) as u32;
            borrows[i] = 1;
            prev_borrow = 1;
        }
    }
    assert_eq!(borrows[11], 0);
    (res, borrows)
}

pub fn mul_u32_slice_u32(x: &[u32; 12], y: u32) -> ([u32; 12], [u32; 12]) {
    let mut prev_carry = 0u32;
    let mut res = [0u32; 12];
    let mut carries = [0u32; 12];
    for i in 0..12 {
        let tmp = x[i] as u64 * y as u64 + prev_carry as u64;
        res[i] = tmp as u32;
        carries[i] = (tmp >> 32) as u32;
        prev_carry = carries[i];
    }
    assert_eq!(prev_carry, 0);
    (res, carries)
}

pub fn get_bits_as_array(number: u32) -> [u32; 32] {
    let mut result = [0u32; 32]; // Assuming a u32 has 32 bits

    for i in 0..32 {
        // Shift the 1 bit to the rightmost position and perform bitwise AND
        result[i] = ((number >> i) & 1) as u32;
    }

    result
}

pub fn add_u32_slices_1(x: &[u32; 24], y: &[u32; 25]) -> ([u32; 25], [u32; 24]) {
    let mut x_padded = [0u32; 25];
    x_padded[0..24].copy_from_slice(x);
    let mut prev_carry = 0u32;
    let mut res = [0u32; 25];
    let mut carries = [0u32; 24];
    for i in 0..24 {
        let s = x[i] as u64 + y[i] as u64 + prev_carry as u64;
        let sum = s as u32;
        let carry = (s >> 32) as u32;
        prev_carry = carry;
        res[i] = sum;
        carries[i] = carry;
    }
    res[24] = prev_carry;
    (res, carries)
}

pub fn egcd(a: BigUint, b: BigUint) -> BigUint {
    // if a == BigUint::from(0 as u32){
    //     (b, BigUint::from(0 as u32), BigUint::from(1 as u32))
    // } else {
    //     let (g, y, x) = egcd(b.clone()%a.clone(), a.clone());
    //     (g, x - (b.clone()*(y.clone()/a.clone())), y)
    // }
    let mut a_ = BigInt::from_biguint(Sign::Plus, a);
    let mut b_ = BigInt::from_biguint(Sign::Plus, b);

    let mut x = BigInt::from_str("0").unwrap();
    let mut y = BigInt::from_str("1").unwrap();
    let mut u = BigInt::from_str("1").unwrap();
    let mut v = BigInt::from_str("0").unwrap();

    let zero = BigInt::from_str("0").unwrap();
    while a_ != zero {
        let q = b_.clone() / a_.clone();
        let r = b_ % a_.clone();
        let m = x - (u.clone() * q.clone());
        let n = y - (v.clone() * q);
        b_ = a_;
        a_ = r;
        x = u;
        y = v;
        u = m;
        v = n;
    }
    // println!("x {:?}", x);
    let mod_bigint = modulus().to_bigint().unwrap();
    if x < 0.into() {
        ((x % mod_bigint.clone()) + mod_bigint)
            .to_biguint()
            .unwrap()
    } else {
        (x % mod_bigint.clone()).to_biguint().unwrap()
    }
}

pub fn mod_inverse(a: BigUint, m: BigUint) -> BigUint {
    egcd(a, m.clone())
    // x % m
}

pub fn fp4_square(a: Fp2, b: Fp2) -> (Fp2, Fp2) {
    let a2 = a * a;
    let b2 = b * b;
    (b2.mul_by_nonresidue() + a2, ((a + b) * (a + b)) - a2 - b2)
}

pub fn get_u32_vec_from_literal(x: BigUint) -> [u32; 12] {
    let mut x_u32_vec: Vec<u32> = x.to_u32_digits();
    while x_u32_vec.len() != 12 {
        x_u32_vec.push(0 as u32);
    }
    x_u32_vec.try_into().unwrap()
}

pub fn get_u32_vec_from_literal_ref(x: &BigUint) -> [u32; 12] {
    let mut x_u32_vec: Vec<u32> = x.to_u32_digits();
    while x_u32_vec.len() != 12 {
        x_u32_vec.push(0 as u32);
    }
    x_u32_vec.try_into().unwrap()
}

pub fn get_selector_bits_from_u32(x: u32) -> [u32; 12] {
    // assert!(x<=4096);
    let mut res = [0u32; 12];
    let mut val = x.clone();
    for i in 0..12 {
        res[i] = val & 1;
        val = val >> 1;
    }
    res
}

pub fn get_u32_vec_from_literal_24(x: BigUint) -> [u32; 24] {
    let mut x_u32_vec: Vec<u32> = x.to_u32_digits();
    while x_u32_vec.len() != 24 {
        x_u32_vec.push(0 as u32);
    }
    x_u32_vec.try_into().unwrap()
}

pub fn get_u32_vec_from_literal_ref_24(x: &BigUint) -> [u32; 24] {
    let mut x_u32_vec: Vec<u32> = x.to_u32_digits();
    while x_u32_vec.len() != 24 {
        x_u32_vec.push(0 as u32);
    }
    x_u32_vec.try_into().unwrap()
}

pub fn get_div_rem_modulus_from_biguint_12(x: BigUint) -> ([u32; 12], [u32; 12]) {
    let rem = x.clone() % modulus();
    let div = x / modulus();
    (get_u32_vec_from_literal(div), get_u32_vec_from_literal(rem))
}

pub fn calc_qs(x: Fp2, y: Fp2, z: Fp2) -> (Fp2, Fp2, Fp2) {
    let ax = x * z.invert();
    let ay = y * z.invert();

    let qx = ax.clone();
    let qy = ay.clone();
    let qz = Fp2::one();
    (qx, qy, qz)
}

pub fn calc_precomp_stuff_loop0(rx: Fp2, ry: Fp2, rz: Fp2) -> Vec<Fp2> {
    // runs 1 loop subpart 0
    let t0 = ry * ry;
    let t1 = rz * rz;
    let x0 = t1.mul(Fp::get_fp_from_biguint(BigUint::from(3 as u32)));

    let t2 = x0.multiply_by_b();
    let t3 = t2.mul(Fp::get_fp_from_biguint(BigUint::from(3 as u32)));
    let x1 = ry * rz;
    let t4 = x1.mul(Fp::get_fp_from_biguint(BigUint::from(2 as u32)));
    let x2 = t2 - t0;
    let x3 = rx * rx;
    let x4 = x3.mul(Fp::get_fp_from_biguint(BigUint::from(3 as u32)));
    let x5 = -t4;

    let k = mod_inverse(BigUint::from(2 as u32), modulus());

    let x6 = t0 - t3;
    let x7 = rx * ry;
    let x8 = x6 * x7;

    let x9 = t0 + t3;
    let x10 = x9 * Fp::get_fp_from_biguint(k.clone());
    let x11 = x10 * x10;

    let x12 = t2 * t2;
    let x13 = x12 * Fp::get_fp_from_biguint(BigUint::from(3 as u32));

    let new_rx = x8 * Fp::get_fp_from_biguint(k.clone());
    let new_ry = x11 - x13;
    let new_rz = t0 * t4;

    vec![
        new_rx, new_ry, new_rz, t0, t1, x0, t2, t3, x1, t4, x3, x2, x4, x5, x6, x7, x8, x9, x10,
        x11, x12, x13,
    ]
}

pub fn calc_precomp_stuff_loop1(rx: Fp2, ry: Fp2, rz: Fp2, qx: Fp2, qy: Fp2) -> Vec<Fp2> {
    let bit1_t0 = qy * rz;
    let bit1_t1 = ry - bit1_t0;
    // println!("bit1_t1__ {:?}", bit1_t1.to_biguint());
    let bit1_t2 = qx * rz;
    let bit1_t3 = rx - bit1_t2;
    // println!("t1__ {:?}", bit1_t3.to_biguint());
    let bit1_t4 = bit1_t1 * qx;
    let bit1_t5 = bit1_t3 * qy;
    let bit1_t6 = bit1_t4 - bit1_t5;
    let bit1_t7 = -bit1_t1;
    // println!("ell_coeff_1_0 {:?}", ell_coeff[1][0].to_biguint());
    // println!("ell_coeff_1_1 {:?}", ell_coeff[1][1].to_biguint());
    // println!("ell_coeff_1_2 {:?}", ell_coeff[1][2].to_biguint());
    let bit1_t8 = bit1_t3 * bit1_t3;
    // println!("t2__ {:?}", bit1_t8.to_biguint());
    let bit1_t9 = bit1_t8 * bit1_t3;
    // println!("t3__ {:?}", bit1_t9.to_biguint());
    let bit1_t10 = bit1_t8 * rx;
    // println!("t4__ {:?}", bit1_t10.to_biguint());
    let bit1_t11 = bit1_t1 * bit1_t1;
    let bit1_t12 = bit1_t11 * rz;
    let bit1_t13 = bit1_t10 * Fp::get_fp_from_biguint(BigUint::from(2 as u32));
    let bit1_t14 = bit1_t9 - bit1_t13;
    let bit1_t15 = bit1_t14 + bit1_t12;
    // println!("t5__ {:?}", bit1_t15.to_biguint());
    let bit1_t16 = bit1_t10 - bit1_t15;
    let bit1_t17 = bit1_t16 * bit1_t1;
    let bit1_t18 = bit1_t9 * ry;
    let new_rx = bit1_t3 * bit1_t15;
    let new_ry = bit1_t17 - bit1_t18;
    let new_rz = rz * bit1_t9;

    vec![
        new_rx, new_ry, new_rz, bit1_t0, bit1_t1, bit1_t2, bit1_t3, bit1_t4, bit1_t5, bit1_t6,
        bit1_t7, bit1_t8, bit1_t9, bit1_t10, bit1_t11, bit1_t12, bit1_t13, bit1_t14, bit1_t15,
        bit1_t16, bit1_t17, bit1_t18,
    ]
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Fp(pub(crate) [u32; 12]);

impl Fp {
    pub fn zero() -> Fp {
        Fp([0; 12])
    }

    pub fn one() -> Fp {
        let mut x = Fp([0; 12]);
        x.0[0] = 1;
        x
    }

    pub fn get_fp_from_biguint(x: BigUint) -> Fp {
        Fp(get_u32_vec_from_literal(x))
    }

    pub fn get_bitlen(&self) -> u64 {
        BigUint::new(self.0.try_into().unwrap()).bits()
    }

    pub fn get_bit(&self, idx: u64) -> bool {
        BigUint::new(self.0.try_into().unwrap()).bit(idx)
    }

    pub fn invert(&self) -> Self {
        let rhs_buint = BigUint::new(self.0.try_into().unwrap());
        let inv = mod_inverse(rhs_buint, modulus());
        // println!("inv {:?}", inv);
        Fp::get_fp_from_biguint(inv)
    }

    pub fn to_biguint(&self) -> BigUint {
        BigUint::new(self.0.to_vec())
    }
}

impl Div for Fp {
    type Output = Self;

    fn div(self, rhs: Self) -> Self::Output {
        let rhs_buint = BigUint::new(rhs.0.try_into().unwrap());
        let inv = mod_inverse(rhs_buint, modulus());
        self * Fp::get_fp_from_biguint(inv)
    }
}

impl Add for Fp {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        add_fp(self, rhs)
    }
}

impl Mul for Fp {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        // let x_b = BigUint::new(self.0.try_into().unwrap());
        // let y_b = BigUint::new(rhs.0.try_into().unwrap());
        // let z = (x_b * y_b).modpow(&BigUint::from_str("1").unwrap(), &modulus());
        // Fp(get_u32_vec_from_literal(z))
        mul_fp(self, rhs)
    }
}

impl Neg for Fp {
    type Output = Self;

    fn neg(self) -> Self::Output {
        let x: BigUint = BigUint::new(self.0.try_into().unwrap());
        Fp(get_u32_vec_from_literal(modulus() - x))
    }
}

impl Sub for Fp {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        // let x_b = BigUint::new(self.0.try_into().unwrap());
        // let y_b = BigUint::new(rhs.0.try_into().unwrap());
        // let z = (x_b - y_b + modulus()).modpow(&BigUint::from_str("1").unwrap(), &modulus());
        // Fp(get_u32_vec_from_literal(z))
        sub_fp(self, rhs)
    }
}

pub fn add_fp(x: Fp, y: Fp) -> Fp {
    // let x_b = BigUint::new(x.0.try_into().unwrap());
    // let y_b = BigUint::new(y.0.try_into().unwrap());
    // let z = (x_b + y_b).modpow(&BigUint::from_str("1").unwrap(), &modulus());
    // Fp(get_u32_vec_from_literal(z))
    let x_plus_y = big_add(&x.0, &y.0);
    let mut m = modulus_digits();
    m.push(0);
    if big_less_than(&x_plus_y, &m) {
        Fp(x_plus_y[..12].try_into().unwrap())
    } else {
        let (x_plus_y_reduce, _) = big_arithmetic::big_sub(&x_plus_y, &m);
        Fp(x_plus_y_reduce[..12].try_into().unwrap())
    }
    // todo!()
}

pub fn add_fp_without_reduction(x: Fp, y: Fp) -> [u32; 12] {
    // let x_b = BigUint::new(x.0.try_into().unwrap());
    // let y_b = BigUint::new(y.0.try_into().unwrap());
    // let z = (x_b + y_b).modpow(&BigUint::from_str("1").unwrap(), &modulus());
    // Fp(get_u32_vec_from_literal(z))
    let x_plus_y = big_add(&x.0, &y.0);
    get_u32_vec_from_literal(BigUint::new(x_plus_y))
    // todo!()
}

pub fn mul_fp(x: Fp, y: Fp) -> Fp {
    //println!("sub_fp x{:?}, y{:?}", x, y);
    let x_b = BigUint::new(x.0.try_into().unwrap());
    let y_b = BigUint::new(y.0.try_into().unwrap());
    let z = (x_b * y_b).modpow(&BigUint::from_str("1").unwrap(), &modulus());
    //println!("z {:?} {:?}", z.to_u32_digits(), z.to_u32_digits().len());
    Fp(get_u32_vec_from_literal(z))
}

pub fn mul_fp_without_reduction(x: Fp, y: Fp) -> [u32; 24] {
    let x_b = BigUint::new(x.0.try_into().unwrap());
    let y_b = BigUint::new(y.0.try_into().unwrap());
    let z = x_b * y_b;
    get_u32_vec_from_literal_24(z)
}

pub fn negate_fp(x: Fp) -> Fp {
    let x: BigUint = BigUint::new(x.0.try_into().unwrap());
    Fp(get_u32_vec_from_literal(modulus() - x))
}

pub fn sub_fp(x: Fp, y: Fp) -> Fp {
    // println!("sub_fp x{:?}, y{:?}", x, y);
    let x_b = BigUint::new(x.0.try_into().unwrap());
    let y_b = BigUint::new(y.0.try_into().unwrap());
    let z = (modulus() + x_b - y_b).modpow(&BigUint::from_str("1").unwrap(), &modulus());
    // println!("sub_fp::{:?}-{:?}",z.to_u32_digits(), z.to_u32_digits().len());
    Fp(get_u32_vec_from_literal(z))
}

pub fn sum_of_products(a: Vec<Fp>, b: Vec<Fp>) -> Fp {
    let acc = a.iter().zip(b.iter()).fold(Fp([0; 12]), |acc, (a_i, b_i)| {
        add_fp(mul_fp(a_i.clone(), b_i.clone()), acc)
    });
    acc
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Fp2(pub [Fp; 2]);

impl Fp2 {
    pub fn zero() -> Fp2 {
        Fp2([Fp::zero(), Fp::zero()])
    }

    pub fn one() -> Fp2 {
        Fp2([Fp::one(), Fp::zero()])
    }

    pub fn non_residue() -> Fp {
        Fp::get_fp_from_biguint(modulus() - BigUint::from(1 as u32))
    }

    pub fn multiply_by_b(&self) -> Fp2 {
        let t0 = self.0[0].mul(Fp::get_fp_from_biguint(BigUint::from(4 as u32)));
        let t1 = self.0[1].mul(Fp::get_fp_from_biguint(BigUint::from(4 as u32)));
        Fp2([t0 - t1, t0 + t1])
    }

    pub fn mul_by_nonresidue(&self) -> Self {
        let c0 = self.0[0];
        let c1 = self.0[1];
        Fp2([c0 - c1, c0 + c1])
    }

    pub fn invert(&self) -> Self {
        let re = self.0[0];
        let im = self.0[1];
        let factor_fp = (re * re) + (im * im);
        let factor = factor_fp.invert();
        Fp2([factor * re, factor * (-im)])
    }

    pub fn to_biguint(&self) -> [BigUint; 2] {
        [
            BigUint::new(self.0[0].0.to_vec()),
            BigUint::new(self.0[1].0.to_vec()),
        ]
    }

    pub fn get_u32_slice(&self) -> [[u32; 12]; 2] {
        [self.0[0].0, self.0[1].0]
    }
}

impl Add for Fp2 {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        add_fp2(self, rhs)
    }
}

impl Mul for Fp2 {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        mul_fp2(self, rhs)
    }
}

impl Sub for Fp2 {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        sub_fp2(self, rhs)
    }
}

impl Div for Fp2 {
    type Output = Self;
    fn div(self, rhs: Self) -> Self::Output {
        self * rhs.invert()
    }
}

impl Fp2 {
    pub fn roots_of_unity_8th() -> Vec<Fp2> {
        vec![
            Fp2([Fp::one(), Fp::zero()]),
            Fp2([Fp::zero(), Fp::one()]),
            Fp2([Fp::get_fp_from_biguint(BigUint::from_str(
                "1028732146235106349975324479215795277384839936929757896155643118032610843298655225875571310552543014690878354869257"
            ).unwrap()); 2]),
            Fp2([
                Fp::get_fp_from_biguint(BigUint::from_str(
                    "1028732146235106349975324479215795277384839936929757896155643118032610843298655225875571310552543014690878354869257"
                ).unwrap()),
                Fp::get_fp_from_biguint(BigUint::from_str(
                    "2973677408986561043442465346520108879172042883009249989176415018091420807192182638567116318576472649347015917690530"
                ).unwrap()),
            ])
        ]
    }

    pub fn etas() -> Vec<Fp2> {
        vec![
            Fp2([
                Fp::get_fp_from_biguint(BigUint::from_str(
                    "1015919005498129635886032702454337503112659152043614931979881174103627376789972962005013361970813319613593700736144"
                ).unwrap()),
                Fp::get_fp_from_biguint(BigUint::from_str(
                    "1244231661155348484223428017511856347821538750986231559855759541903146219579071812422210818684355842447591283616181"
                ).unwrap()),
            ]),
            Fp2([
                Fp::get_fp_from_biguint(BigUint::from_str(
                    "2758177894066318909194361808224047808735344068952776325476298594220885430911766052020476810444659821590302988943606"
                ).unwrap()),
                Fp::get_fp_from_biguint(BigUint::from_str(
                    "1015919005498129635886032702454337503112659152043614931979881174103627376789972962005013361970813319613593700736144"
                ).unwrap()),
            ]),
            Fp2([
                Fp::get_fp_from_biguint(BigUint::from_str(
                    "1646015993121829755895883253076789309308090876275172350194834453434199515639474951814226234213676147507404483718679"
                ).unwrap()),
                Fp::get_fp_from_biguint(BigUint::from_str(
                    "1637752706019426886789797193293828301565549384974986623510918743054325021588194075665960171838131772227885159387073"
                ).unwrap()),
            ]),
            Fp2([
                Fp::get_fp_from_biguint(BigUint::from_str(
                    "2364656849202240506627992632442075854991333434964021261821139393069706628902643788776727457290883891810009113172714"
                ).unwrap()),
                Fp::get_fp_from_biguint(BigUint::from_str(
                    "1646015993121829755895883253076789309308090876275172350194834453434199515639474951814226234213676147507404483718679"
                ).unwrap()),
            ]),
        ]
    }
}

impl Mul<Fp> for Fp2 {
    type Output = Fp2;

    fn mul(self, rhs: Fp) -> Self::Output {
        // let mut ans = Fp2::zero();
        // let mut found_one = false;
        // for i in (0..rhs.get_bitlen()).rev() {
        //     if found_one {
        //         ans = ans + ans;
        //     }
        //     let bit  = rhs.get_bit(i);
        //     if bit {
        //         found_one = true;
        //         ans = ans + self;
        //     }
        // }
        let fp2 = self.0;

        let ans = Fp2([fp2[0] * rhs, fp2[1] * rhs]);
        ans
    }
}

impl Neg for Fp2 {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Fp2([self.0[0].neg(), self.0[1].neg()])
    }
}

pub fn sub_fp2(x: Fp2, y: Fp2) -> Fp2 {
    Fp2([sub_fp(x.0[0], y.0[0]), sub_fp(x.0[1], y.0[1])])
}

pub fn add_fp2(x: Fp2, y: Fp2) -> Fp2 {
    Fp2([add_fp(x.0[0], y.0[0]), add_fp(x.0[1], y.0[1])])
}

pub fn mul_fp2(x: Fp2, y: Fp2) -> Fp2 {
    //println!("x:: {:?}", x);
    //println!("y:: {:?}", y);
    let c0 = sub_fp(mul_fp(x.0[0], y.0[0]), mul_fp(x.0[1], y.0[1]));
    let c1 = add_fp(mul_fp(x.0[0], y.0[1]), mul_fp(x.0[1], y.0[0]));
    Fp2([c0, c1])
}

// pub fn mul_fp2_without_reduction(x: Fp2, y: Fp2) -> Fp2 {

// }

#[derive(Clone, Copy, Debug)]
pub struct Fp6(pub(crate) [Fp; 6]);

impl Fp6 {
    pub fn invert(&self) -> Self {
        let c0c1c2 = self;
        let c0 = Fp2(c0c1c2.0[0..2].to_vec().try_into().unwrap());
        let c1 = Fp2(c0c1c2.0[2..4].to_vec().try_into().unwrap());
        let c2 = Fp2(c0c1c2.0[4..6].to_vec().try_into().unwrap());
        let t0 = (c0 * c0) - (c2 * c1).mul_by_nonresidue();
        let t1 = (c2 * c2).mul_by_nonresidue() - (c0 * c1);
        let t2 = (c1 * c1) - (c0 * c2);
        let t4 = (((c2 * t1) + (c1 * t2)).mul_by_nonresidue() + (c0 * t0)).invert();
        Fp6([(t4 * t0).0, (t4 * t1).0, (t4 * t2).0]
            .concat()
            .try_into()
            .unwrap())
    }

    pub fn get_u32_slice(&self) -> [[u32; 12]; 6] {
        self.0
            .iter()
            .map(|f| f.0)
            .collect::<Vec<[u32; 12]>>()
            .try_into()
            .unwrap()
    }

    pub fn print(&self) {
        // println!("--- Printing Fp6 ---");
        // for i in 0..self.0.len() {
        //     let fp = Fp::get_fp_from_biguint(BigUint::new(self.0[i].0.to_vec()));
        //     println!("i -- {:?}",fp.to_biguint());
        // }
        // println!("--- Printed Fp6 ---");
    }
}

impl Add for Fp6 {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        add_fp6(self, rhs)
    }
}

impl Sub for Fp6 {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        sub_fp6(self, rhs)
    }
}

impl Div for Fp6 {
    type Output = Self;

    fn div(self, rhs: Self) -> Self::Output {
        self * rhs.invert()
    }
}

impl Mul for Fp6 {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        mul_fp6(self, rhs)
    }
}

impl Neg for Fp6 {
    type Output = Self;

    fn neg(self) -> Self::Output {
        let c0c1c2 = self;
        let c0 = Fp2(c0c1c2.0[0..2].to_vec().try_into().unwrap());
        let c1 = Fp2(c0c1c2.0[2..4].to_vec().try_into().unwrap());
        let c2 = Fp2(c0c1c2.0[4..6].to_vec().try_into().unwrap());
        Fp6([c0.neg().0, c1.neg().0, c2.neg().0]
            .concat()
            .try_into()
            .unwrap())
    }
}

pub fn add_fp6(x: Fp6, y: Fp6) -> Fp6 {
    Fp6([
        add_fp(x.0[0], y.0[0]),
        add_fp(x.0[1], y.0[1]),
        add_fp(x.0[2], y.0[2]),
        add_fp(x.0[3], y.0[3]),
        add_fp(x.0[4], y.0[4]),
        add_fp(x.0[5], y.0[5]),
    ])
}

pub fn sub_fp6(x: Fp6, y: Fp6) -> Fp6 {
    Fp6([
        sub_fp(x.0[0], y.0[0]),
        sub_fp(x.0[1], y.0[1]),
        sub_fp(x.0[2], y.0[2]),
        sub_fp(x.0[3], y.0[3]),
        sub_fp(x.0[4], y.0[4]),
        sub_fp(x.0[5], y.0[5]),
    ])
}
/*
Fp6 -> Fp2(c0), c1, c2

    [c0.c0, c0.c1, c1.c0, c1.c1, c2.c0, c2.c1]
 */
pub fn mul_fp6(x: Fp6, y: Fp6) -> Fp6 {
    let c0 = Fp2([x.0[0], x.0[1]]);
    let c1 = Fp2([x.0[2], x.0[3]]);
    let c2 = Fp2([x.0[4], x.0[5]]);

    let r0 = Fp2([y.0[0], y.0[1]]);
    let r1 = Fp2([y.0[2], y.0[3]]);
    let r2 = Fp2([y.0[4], y.0[5]]);

    let t0 = c0 * r0;
    let t1 = c1 * r1;
    let t2 = c2 * r2;

    let t3 = c1 + c2;
    let t4 = r1 + r2;
    let t5 = t3 * t4;
    let t6 = t5 - t1;
    let t7 = t6 - t2;
    let t8 = t7.mul_by_nonresidue();
    let x = t8 + t0;

    let t9 = c0 + c1;
    let t10 = r0 + r1;
    let t11 = t9 * t10;
    let t12 = t11 - t0;
    let t13 = t12 - t1;
    let t14 = t2.mul_by_nonresidue();
    let y = t13 + t14;

    let t15 = c0 + c2;
    let t16 = r0 + r2;
    let t17 = t15 * t16;
    let t18 = t17 - t0;
    let t19 = t18 - t2;
    let z = t19 + t1;

    Fp6([x.0[0], x.0[1], y.0[0], y.0[1], z.0[0], z.0[1]])
}

pub fn mul_by_nonresidue(x: [Fp; 6]) -> Fp6 {
    let mut ans: [Fp; 6] = [Fp::zero(); 6];
    let c0 = Fp2([x[4], x[5]]).mul_by_nonresidue();
    ans[0] = c0.0[0];
    ans[1] = c0.0[1];
    ans[2] = x[0];
    ans[3] = x[1];
    ans[4] = x[2];
    ans[5] = x[3];
    Fp6(ans)
}

impl Fp6 {
    pub fn multiply_by_01(&self, b0: Fp2, b1: Fp2) -> Self {
        let c0 = Fp2(self.0[0..2].to_vec().try_into().unwrap());
        let c1 = Fp2(self.0[2..4].to_vec().try_into().unwrap());
        let c2 = Fp2(self.0[4..6].to_vec().try_into().unwrap());

        let t0 = c0 * b0;
        let t1 = c1 * b1;

        let t2 = c2 * b1;
        let t3 = t2.mul_by_nonresidue();
        let x = t3 + t0;

        let t4 = b0 + b1;
        let t5 = c0 + c1;
        let t6 = t4 * t5;
        let t7 = t6 - t0;
        let y = t7 - t1;

        let t8 = c2 * b0;
        let z = t8 + t1;
        Fp6([x.0, y.0, z.0].concat().try_into().unwrap())
    }

    pub fn multiply_by_1(&self, b1: Fp2) -> Self {
        let c0 = Fp2(self.0[0..2].to_vec().try_into().unwrap());
        let c1 = Fp2(self.0[2..4].to_vec().try_into().unwrap());
        let c2 = Fp2(self.0[4..6].to_vec().try_into().unwrap());

        let t0 = c2 * b1;
        let x = t0.mul_by_nonresidue();

        let y = c0 * b1;

        let z = c1 * b1;
        Fp6([x.0, y.0, z.0].concat().try_into().unwrap())
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Fp12(pub(crate) [Fp; 12]);

impl Fp12 {
    pub fn one() -> Fp12 {
        let mut x = [Fp::zero(); 12];
        x[0] = Fp::one();
        Fp12(x)
    }

    pub fn invert(&self) -> Self {
        let c0 = Fp6(self.0[0..6].try_into().unwrap());
        let c1 = Fp6(self.0[6..12].try_into().unwrap());
        let t = (c0 * c0 - mul_by_nonresidue((c1 * c1).0)).invert();
        Fp12([(c0 * t).0, (-(c1 * t)).0].concat().try_into().unwrap())
    }

    pub fn print(&self) {
        // println!("--- Printing Fp12 ---");
        // for i in 0..self.0.len() {
        //     let fp = Fp::get_fp_from_biguint(BigUint::new(self.0[i].0.to_vec()));
        //     println!("i -- {:?}",fp.to_biguint());
        // }
        // println!("--- Printed Fp12 ---");
    }

    pub fn from_str(x: [&str; 12]) -> Self {
        let mut ans: Fp12 = Fp12::one();
        for i in 0..12 {
            let bu = Fp::get_fp_from_biguint(BigUint::from_str(x[i]).unwrap());
            ans.0[i] = bu;
        }
        ans
    }

    pub fn get_u32_slice(&self) -> [[u32; 12]; 12] {
        self.0
            .iter()
            .map(|f| f.0)
            .collect::<Vec<[u32; 12]>>()
            .try_into()
            .unwrap()
    }
}

impl Add for Fp12 {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        add_fp12(self, rhs)
    }
}

impl Mul for Fp12 {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        mul_fp_12(self, rhs)
    }
}

impl Div for Fp12 {
    type Output = Self;

    fn div(self, rhs: Self) -> Self::Output {
        self * rhs.invert()
    }
}

impl Neg for Fp12 {
    type Output = Self;

    fn neg(self) -> Self::Output {
        todo!()
    }
}

// impl Debug for Fp12 {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         f.debug_tuple("Fp12").field(&self.0).finish()
//     }
// }

pub fn add_fp12(x: Fp12, y: Fp12) -> Fp12 {
    let mut ans: [Fp; 12] = [Fp::zero(); 12];
    for i in 0..12 {
        ans[i] = add_fp(x.0[i], y.0[i]);
    }
    Fp12(ans)
}

pub fn mul_fp_12(x: Fp12, y: Fp12) -> Fp12 {
    let c0 = Fp6(x.0[0..6].try_into().unwrap());
    let c1 = Fp6(x.0[6..12].try_into().unwrap());
    let r0 = Fp6(y.0[0..6].try_into().unwrap());
    let r1 = Fp6(y.0[6..12].try_into().unwrap());

    let t0 = c0 * r0;
    let t1 = c1 * r1;
    let t2 = mul_by_nonresidue(t1.0);
    let x = t0 + t2;

    let t3 = c0 + c1;
    let t4 = r0 + r1;
    let t5 = t3 * t4;
    let t6 = t5 - t0;
    let y = t6 - t1;

    Fp12([x.0, y.0].concat().try_into().unwrap())
}

pub trait Pow
where
    Self: Copy + Mul<Output = Self>,
{
    fn pow(&self, one: Self, exp: BigUint) -> Self {
        if exp == 0u32.into() {
            return one;
        }
        if exp == 1u32.into() {
            return *self;
        }
        if exp.clone() % 2u32 == 1u32.into() {
            return *self * self.pow(one, exp - 1u32);
        }
        let d = self.pow(one, exp >> 1);
        d * d
    }
}

impl<T> Pow for T where T: Copy + Mul<Output = Self> {}

impl Fp2 {
    pub fn forbenius_coefficients() -> [Fp; 2] {
        [
            Fp::get_fp_from_biguint(BigUint::from_str("1").unwrap()),
            Fp::get_fp_from_biguint(BigUint::from_str("4002409555221667393417789825735904156556882819939007885332058136124031650490837864442687629129015664037894272559786").unwrap()),
        ]
    }
    pub fn forbenius_map(&self, pow: usize) -> Self {
        let constants = Fp2::forbenius_coefficients();
        Fp2([self.0[0], self.0[1] * constants[pow % 2]])
    }
}

impl Fp6 {
    pub fn forbenius_coefficients_1() -> [Fp2; 6] {
        [
            Fp2([
                Fp::get_fp_from_biguint(BigUint::from_str("1").unwrap()),
                Fp::get_fp_from_biguint(BigUint::from_str("0").unwrap()),
            ]),
            Fp2([
                Fp::get_fp_from_biguint(BigUint::from_str("0").unwrap()),
                Fp::get_fp_from_biguint(BigUint::from_str("4002409555221667392624310435006688643935503118305586438271171395842971157480381377015405980053539358417135540939436").unwrap()),
            ]),
            Fp2([
                Fp::get_fp_from_biguint(BigUint::from_str("793479390729215512621379701633421447060886740281060493010456487427281649075476305620758731620350").unwrap()),
                Fp::get_fp_from_biguint(BigUint::from_str("0").unwrap()),
            ]),
            Fp2([
                Fp::get_fp_from_biguint(BigUint::from_str("0").unwrap()),
                Fp::get_fp_from_biguint(BigUint::from_str("1").unwrap()),
            ]),
            Fp2([
                Fp::get_fp_from_biguint(BigUint::from_str("793479390729215512621379701633421447060886740281060493010456487427281649075476305620758731620350").unwrap()),
                Fp::get_fp_from_biguint(BigUint::from_str("0").unwrap()),
            ]),
            Fp2([
                Fp::get_fp_from_biguint(BigUint::from_str("0").unwrap()),
                Fp::get_fp_from_biguint(BigUint::from_str("4002409555221667392624310435006688643935503118305586438271171395842971157480381377015405980053539358417135540939436").unwrap()),
            ]),
        ]
    }

    pub fn forbenius_coefficients_2() -> [Fp2; 6] {
        [
            Fp2([
                Fp::get_fp_from_biguint(BigUint::from_str("1").unwrap()),
                Fp::get_fp_from_biguint(BigUint::from_str("0").unwrap()),
            ]),
            Fp2([
                Fp::get_fp_from_biguint(BigUint::from_str("4002409555221667392624310435006688643935503118305586438271171395842971157480381377015405980053539358417135540939437").unwrap()),
                Fp::get_fp_from_biguint(BigUint::from_str("0").unwrap()),
            ]),
            Fp2([
                Fp::get_fp_from_biguint(BigUint::from_str("4002409555221667392624310435006688643935503118305586438271171395842971157480381377015405980053539358417135540939436").unwrap()),
                Fp::get_fp_from_biguint(BigUint::from_str("0").unwrap()),
            ]),
            Fp2([
                Fp::get_fp_from_biguint(BigUint::from_str("4002409555221667393417789825735904156556882819939007885332058136124031650490837864442687629129015664037894272559786").unwrap()),
                Fp::get_fp_from_biguint(BigUint::from_str("0").unwrap()),
            ]),
            Fp2([
                Fp::get_fp_from_biguint(BigUint::from_str("793479390729215512621379701633421447060886740281060493010456487427281649075476305620758731620350").unwrap()),
                Fp::get_fp_from_biguint(BigUint::from_str("0").unwrap()),
            ]),
            Fp2([
                Fp::get_fp_from_biguint(BigUint::from_str("0").unwrap()),
                Fp::get_fp_from_biguint(BigUint::from_str("793479390729215512621379701633421447060886740281060493010456487427281649075476305620758731620351").unwrap()),
            ]),
        ]
    }
    pub fn forbenius_map(&self, pow: usize) -> Self {
        // println!("--- fp6 forbenius map ---");
        let fp6_frobenius_coefficients_1 = Fp6::forbenius_coefficients_1();

        let fp6_frobenius_coefficients_2 = Fp6::forbenius_coefficients_2();
        self.print();
        let c0 = Fp2(self.0[0..2].to_vec().try_into().unwrap());
        // println!("c0 {:?}", c0.to_biguint());
        let c1 = Fp2(self.0[2..4].to_vec().try_into().unwrap());
        // println!("c1 {:?}", c0.to_biguint());
        let c2 = Fp2(self.0[4..6].to_vec().try_into().unwrap());
        // println!("c2 {:?}", c0.to_biguint());
        // println!("--- fp6 forbenius map ---");
        Fp6([
            c0.forbenius_map(pow).0,
            (c1.forbenius_map(pow) * fp6_frobenius_coefficients_1[pow % 6]).0,
            (c2.forbenius_map(pow) * fp6_frobenius_coefficients_2[pow % 6]).0,
        ]
        .concat()
        .try_into()
        .unwrap())
    }
}

impl Fp12 {
    pub fn forbenius_coefficients() -> [Fp2; 12] {
        [
            Fp2([
                Fp::get_fp_from_biguint(BigUint::from_str("1").unwrap()),
                Fp::get_fp_from_biguint(BigUint::from_str("0").unwrap()),
            ]),
            Fp2([
                Fp::get_fp_from_biguint(BigUint::from_str("3850754370037169011952147076051364057158807420970682438676050522613628423219637725072182697113062777891589506424760").unwrap()),
                Fp::get_fp_from_biguint(BigUint::from_str("151655185184498381465642749684540099398075398968325446656007613510403227271200139370504932015952886146304766135027").unwrap()),
            ]),
            Fp2([
                Fp::get_fp_from_biguint(BigUint::from_str("793479390729215512621379701633421447060886740281060493010456487427281649075476305620758731620351").unwrap()),
                Fp::get_fp_from_biguint(BigUint::from_str("0").unwrap()),
            ]),
            Fp2([
                Fp::get_fp_from_biguint(BigUint::from_str("2973677408986561043442465346520108879172042883009249989176415018091420807192182638567116318576472649347015917690530").unwrap()),
                Fp::get_fp_from_biguint(BigUint::from_str("1028732146235106349975324479215795277384839936929757896155643118032610843298655225875571310552543014690878354869257").unwrap()),
            ]),
            Fp2([
                Fp::get_fp_from_biguint(BigUint::from_str("793479390729215512621379701633421447060886740281060493010456487427281649075476305620758731620350").unwrap()),
                Fp::get_fp_from_biguint(BigUint::from_str("0").unwrap()),
            ]),
            Fp2([
                Fp::get_fp_from_biguint(BigUint::from_str("3125332594171059424908108096204648978570118281977575435832422631601824034463382777937621250592425535493320683825557").unwrap()),
                Fp::get_fp_from_biguint(BigUint::from_str("877076961050607968509681729531255177986764537961432449499635504522207616027455086505066378536590128544573588734230").unwrap()),
            ]),
            Fp2([
                Fp::get_fp_from_biguint(BigUint::from_str("4002409555221667393417789825735904156556882819939007885332058136124031650490837864442687629129015664037894272559786").unwrap()),
                Fp::get_fp_from_biguint(BigUint::from_str("0").unwrap()),
            ]),
            Fp2([
                Fp::get_fp_from_biguint(BigUint::from_str("151655185184498381465642749684540099398075398968325446656007613510403227271200139370504932015952886146304766135027").unwrap()),
                Fp::get_fp_from_biguint(BigUint::from_str("3850754370037169011952147076051364057158807420970682438676050522613628423219637725072182697113062777891589506424760").unwrap()),
            ]),
            Fp2([
                Fp::get_fp_from_biguint(BigUint::from_str("4002409555221667392624310435006688643935503118305586438271171395842971157480381377015405980053539358417135540939436").unwrap()),
                Fp::get_fp_from_biguint(BigUint::from_str("0").unwrap()),
            ]),
            Fp2([
                Fp::get_fp_from_biguint(BigUint::from_str("1028732146235106349975324479215795277384839936929757896155643118032610843298655225875571310552543014690878354869257").unwrap()),
                Fp::get_fp_from_biguint(BigUint::from_str("2973677408986561043442465346520108879172042883009249989176415018091420807192182638567116318576472649347015917690530").unwrap()),
            ]),
            Fp2([
                Fp::get_fp_from_biguint(BigUint::from_str("4002409555221667392624310435006688643935503118305586438271171395842971157480381377015405980053539358417135540939437").unwrap()),
                Fp::get_fp_from_biguint(BigUint::from_str("0").unwrap()),
            ]),
            Fp2([
                Fp::get_fp_from_biguint(BigUint::from_str("877076961050607968509681729531255177986764537961432449499635504522207616027455086505066378536590128544573588734230").unwrap()),
                Fp::get_fp_from_biguint(BigUint::from_str("3125332594171059424908108096204648978570118281977575435832422631601824034463382777937621250592425535493320683825557").unwrap()),
            ]),
        ]
    }

    pub fn forbenius_map(&self, pow: usize) -> Self {
        // println!(" ---- forbenius - map -----");
        let fp12_forbenius_coefficients = Fp12::forbenius_coefficients();
        let r0 = Fp6(self.0[0..6].to_vec().try_into().unwrap()).forbenius_map(pow);
        r0.print();
        let c0c1c2 = Fp6(self.0[6..12].to_vec().try_into().unwrap()).forbenius_map(pow);
        c0c1c2.print();
        let c0 = Fp2(c0c1c2.0[0..2].to_vec().try_into().unwrap());
        // println!("c0 - {:?}", c0.to_biguint());
        let c1 = Fp2(c0c1c2.0[2..4].to_vec().try_into().unwrap());
        // println!("c1 - {:?}", c1.to_biguint());
        let c2 = Fp2(c0c1c2.0[4..6].to_vec().try_into().unwrap());
        // println!("c2 - {:?}", c2.to_biguint());
        let coeff = fp12_forbenius_coefficients[pow % 12];
        // println!("coeff - {:?}", coeff.to_biguint());
        Fp12(
            [
                r0.0,
                [(c0 * coeff).0, (c1 * coeff).0, (c2 * coeff).0]
                    .concat()
                    .try_into()
                    .unwrap(),
            ]
            .concat()
            .try_into()
            .unwrap(),
        )
    }
}

impl Fp12 {
    pub fn multiply_by_014(&self, o0: Fp2, o1: Fp2, o4: Fp2) -> Self {
        let c0 = Fp6(self.0[0..6].to_vec().try_into().unwrap());
        let c1 = Fp6(self.0[6..12].to_vec().try_into().unwrap());
        let t0 = c0.multiply_by_01(o0, o1);
        let t1 = c1.multiply_by_1(o4);
        let t2 = mul_by_nonresidue(t1.0);
        let x = t2 + t0;

        let t3 = c1 + c0;
        let t4 = o1 + o4;
        let t5 = t3.multiply_by_01(o0, t4);
        let t6 = t5 - t0;
        let y = t6 - t1;
        Fp12([x.0, y.0].concat().try_into().unwrap())
    }

    pub fn conjugate(&self) -> Self {
        let mut x = self.0.clone();
        for i in 6..12 {
            x[i] = -x[i];
        }
        Fp12(x)
    }

    pub fn cyclotomic_square(&self) -> Self {
        let two = Fp::get_fp_from_biguint(BigUint::from(2 as u32));

        let c0c0 = Fp2(self.0[0..2].try_into().unwrap());
        let c0c1 = Fp2(self.0[2..4].try_into().unwrap());
        let c0c2 = Fp2(self.0[4..6].try_into().unwrap());
        let c1c0 = Fp2(self.0[6..8].try_into().unwrap());
        let c1c1 = Fp2(self.0[8..10].try_into().unwrap());
        let c1c2 = Fp2(self.0[10..12].try_into().unwrap());

        let t0 = fp4_square(c0c0, c1c1);
        let t1 = fp4_square(c1c0, c0c2);
        let t2 = fp4_square(c0c1, c1c2);
        let t3 = t2.1.mul_by_nonresidue();

        let t4 = t0.0 - c0c0;
        let t5 = t4 * two;
        let c0 = t5 + t0.0;

        let t6 = t1.0 - c0c1;
        let t7 = t6 * two;
        let c1 = t7 + t1.0;

        let t8 = t2.0 - c0c2;
        let t9 = t8 * two;
        let c2 = t9 + t2.0;

        let t10 = t3 + c1c0;
        let t11 = t10 * two;
        let c3 = t11 + t3;

        let t12 = t0.1 + c1c1;
        let t13 = t12 * two;
        let c4 = t13 + t0.1;

        let t14 = t1.1 + c1c2;
        let t15 = t14 * two;
        let c5 = t15 + t1.1;

        Fp12(
            [c0.0, c1.0, c2.0, c3.0, c4.0, c5.0]
                .concat()
                .try_into()
                .unwrap(),
        )
    }

    pub fn cyclotocmic_exponent(&self) -> Fp12 {
        let mut z = Fp12::one();
        for i in (0..get_bls_12_381_parameter().bits()).rev() {
            z = z.cyclotomic_square();
            if get_bls_12_381_parameter().bit(i) {
                z = z * self.clone();
            }
        }
        z
    }

    pub fn final_exponentiate(&self) -> Self {
        let t_0 = self.forbenius_map(6);
        let t_1 = t_0 / self.clone();
        let t_2 = t_1.forbenius_map(2);
        let t_3 = t_2 * t_1;
        let t_4 = t_3.cyclotocmic_exponent();
        let t_5 = t_4.conjugate();
        let t_6 = t_3.cyclotomic_square();
        let t_7 = t_6.conjugate();
        let t_8 = t_7 * t_5;
        let t_9 = t_8.cyclotocmic_exponent();
        let t_10 = t_9.conjugate();
        let t_11 = t_10.cyclotocmic_exponent();
        let t_12 = t_11.conjugate();
        let t_13 = t_12.cyclotocmic_exponent();
        let t_14 = t_13.conjugate();
        let t_15 = t_5.cyclotomic_square();
        let t_16 = t_14 * t_15;
        let t_17 = t_16.cyclotocmic_exponent();
        let t_18 = t_17.conjugate();
        let t_19 = t_5 * t_12;
        let t_20 = t_19.forbenius_map(2);
        let t_21 = t_10 * t_3;
        let t_22 = t_21.forbenius_map(3);
        let t_23 = t_3.conjugate();
        let t_24 = t_16 * t_23;
        let t_25 = t_24.forbenius_map(1);
        let t_26 = t_8.conjugate();
        let t_27 = t_18 * t_26;
        let t_28 = t_27 * t_3;
        let t_29 = t_20 * t_22;
        let t_30 = t_29 * t_25;
        let t_31 = t_30 * t_28;
        t_31
    }
}

pub fn inverse_fp2(x: Fp2) -> Fp2 {
    let t0 = x.0[0] * x.0[0];
    let t1 = x.0[1] * x.0[1];
    let t2 = t0 - (t1 * Fp2::non_residue());
    let t3 = Fp::one() / t2;
    Fp2([x.0[0] * t3, -(x.0[1] * t3)])
}

pub fn calc_pairing_precomp(x: Fp2, y: Fp2, z: Fp2) -> Vec<[Fp2; 3]> {
    let ax = x * (z.invert());
    let ay = y * (z.invert());

    let qx = ax.clone();
    let qy = ay.clone();
    let qz = Fp2::one();

    let mut rx = qx.clone();
    let mut ry = qy.clone();
    let mut rz = qz.clone();

    let mut ell_coeff: Vec<[Fp2; 3]> = Vec::<[Fp2; 3]>::new();

    for i in (0..get_bls_12_381_parameter().bits() - 1).rev() {
        let t0 = ry * ry;
        let t1 = rz * rz;
        let x0 = t1.mul(Fp::get_fp_from_biguint(BigUint::from(3 as u32)));

        let t2 = x0.multiply_by_b();
        let t3 = t2.mul(Fp::get_fp_from_biguint(BigUint::from(3 as u32)));
        let x1 = ry * rz;
        let t4 = x1.mul(Fp::get_fp_from_biguint(BigUint::from(2 as u32)));
        let x2 = t2 - t0;
        let x3 = rx * rx;
        let x4 = x3.mul(Fp::get_fp_from_biguint(BigUint::from(3 as u32)));

        let x5 = -t4;
        ell_coeff.push([x2, x4, x5]);

        let k = mod_inverse(BigUint::from(2 as u32), modulus());

        let x6 = t0 - t3;
        let x7 = rx * ry;
        let x8 = x6 * x7;

        let x9 = t0 + t3;
        let x10 = x9 * Fp::get_fp_from_biguint(k.clone());
        let x11 = x10 * x10;

        let x12 = t2 * t2;
        let x13 = x12 * Fp::get_fp_from_biguint(BigUint::from(3 as u32));

        rx = x8 * Fp::get_fp_from_biguint(k.clone());
        ry = x11 - x13;
        rz = t0 * t4;
        if get_bls_12_381_parameter().bit(i) {
            let bit1_t0 = qy * rz;
            let bit1_t1 = ry - bit1_t0;
            let bit1_t2 = qx * rz;
            let bit1_t3 = rx - bit1_t2;
            let bit1_t4 = bit1_t1 * qx;
            let bit1_t5 = bit1_t3 * qy;
            let bit1_t6 = bit1_t4 - bit1_t5;
            let bit1_t7 = -bit1_t1;
            ell_coeff.push([bit1_t6, bit1_t7, bit1_t3]);
            let bit1_t8 = bit1_t3 * bit1_t3;
            let bit1_t9 = bit1_t8 * bit1_t3;
            let bit1_t10 = bit1_t8 * rx;
            let bit1_t11 = bit1_t1 * bit1_t1;
            let bit1_t12 = bit1_t11 * rz;
            let bit1_t13 = bit1_t10 * Fp::get_fp_from_biguint(BigUint::from(2 as u32));
            let bit1_t14 = bit1_t9 - bit1_t13;
            let bit1_t15 = bit1_t14 + bit1_t12;
            rx = bit1_t3 * bit1_t15;
            let bit1_t16 = bit1_t10 - bit1_t15;
            let bit1_t17 = bit1_t16 * bit1_t1;
            let bit1_t18 = bit1_t9 * ry;
            ry = bit1_t17 - bit1_t18;
            rz = rz * bit1_t9;
        }
    }
    return ell_coeff;
}

pub fn miller_loop(g1_x: Fp, g1_y: Fp, g2_x: Fp2, g2_y: Fp2, g2_z: Fp2) -> Fp12 {
    let precomputes = calc_pairing_precomp(g2_x, g2_y, g2_z);
    // for i in 0..precomputes.len() {
    //     println!("{:?} ----", i);
    //     println!("precomputes calculated 1 - {:?}", precomputes[i][0].to_biguint());
    //     println!("precomputes calculated 2 - {:?}", precomputes[i][1].to_biguint());
    //     println!("precomputes calculated 3 - {:?}", precomputes[i][2].to_biguint());
    // }
    // return Fp12::one();
    let px = g1_x.clone();
    let py = g1_y.clone();
    let mut f12 = Fp12::one();
    let mut j = 0;

    for i in (0..get_bls_12_381_parameter().bits() - 1).rev() {
        let ell_coeffs = precomputes[j];
        f12 = f12.multiply_by_014(ell_coeffs[0], ell_coeffs[1] * px, ell_coeffs[2] * py);
        if get_bls_12_381_parameter().bit(i) {
            j += 1;
            let ell_coeffs = precomputes[j];
            f12 = f12.multiply_by_014(ell_coeffs[0], ell_coeffs[1] * px, ell_coeffs[2] * py);
        }
        if i != 0 {
            f12 = mul_fp_12(f12, f12);
        }
        j += 1;
    }
    f12.conjugate()
}

pub fn pairing(p_x: Fp, p_y: Fp, q_x: Fp2, q_y: Fp2, q_z: Fp2) -> Fp12 {
    let looped = miller_loop(p_x, p_y, q_x, q_y, q_z);
    looped
    // looped.final_exponentiate()
}

pub fn verify_bls_signatures() -> bool {
    // Public key
    // Splits into little endian
    let pk_x = BigUint::from_str("2620359726099670991095913421423408052907220385587653382880494211997835858894431070728023161812841650498384724513574").unwrap().to_u32_digits();
    let pk_y = BigUint::from_str("3516737663249789719313994746945990853755171862112391852604784999536233979171013701039178918880615112139780777770781").unwrap().to_u32_digits();
    // Hashed message in g2
    let hm_x1 = BigUint::from_str("2260803321181951703309420903406460477209912434020120381027413359130883713514969717876465885091628521232768207917010").unwrap().to_u32_digits();
    let hm_x2 = BigUint::from_str("2651754974217764549573984422821173864573267897233450902768900290919635595830847280035238812354259899816422437732519").unwrap().to_u32_digits();
    let hm_y1 = BigUint::from_str("98328085801950751198634977711657076320088798571641012335466428770177401024922163125657710674003178075431656844523").unwrap().to_u32_digits();
    let hm_y2 = BigUint::from_str("1156585784149709375944843577113354173925120574246839648967751052400396372157500751188298724114933365921247443786825").unwrap().to_u32_digits();
    let hm_z1 = BigUint::from_str("1").unwrap().to_u32_digits();
    let hm_z2 = BigUint::from_str("0").unwrap().to_u32_digits();
    // Generator
    let gx = BigUint::from_str("3685416753713387016781088315183077757961620795782546409894578378688607592378376318836054947676345821548104185464507").unwrap().to_u32_digits();
    let gy = BigUint::from_str("1339506544944476473020471379941921221584933875938349620426543736416511423956333506472724655353366534992391756441569").unwrap().to_u32_digits();
    // Signature
    let s_x1 = BigUint::from_str("1836830352577417292089156350591626007357750969609299199820146458689304398967104037069103513169938118550765216427090").unwrap().to_u32_digits();
    let s_x2 = BigUint::from_str("2100427494885604888487796981102940167438916035063712025295231442815788486916593575072180414962669967540847907858502").unwrap().to_u32_digits();
    let s_y1 = BigUint::from_str("2555154678035007654633840738122526356989849358171638629627190730328888205299908476410927833296830659413727831906911").unwrap().to_u32_digits();
    let s_y2 = BigUint::from_str("697448450483092846649680958149948400499140883635140106996999493850809967308993531752440334328367413010709405099565").unwrap().to_u32_digits();
    let s_z1 = BigUint::from_str("1").unwrap().to_u32_digits();
    let s_z2 = BigUint::from_str("0").unwrap().to_u32_digits();

    // 1. negate Signature
    let pk_x_negate = pk_x.clone();
    let pk_y_negate = (modulus() - BigUint::new(pk_y)).to_u32_digits();

    let pk_x_neg_fp = Fp::get_fp_from_biguint(BigUint::new(pk_x_negate));
    let pk_y_neg_fp = Fp::get_fp_from_biguint(BigUint::new(pk_y_negate));

    let hmx_fp2 = Fp2([
        Fp::get_fp_from_biguint(BigUint::new(hm_x1)),
        Fp::get_fp_from_biguint(BigUint::new(hm_x2)),
    ]);
    let hmy_fp2 = Fp2([
        Fp::get_fp_from_biguint(BigUint::new(hm_y1)),
        Fp::get_fp_from_biguint(BigUint::new(hm_y2)),
    ]);
    let hmz_fp2 = Fp2([
        Fp::get_fp_from_biguint(BigUint::new(hm_z1)),
        Fp::get_fp_from_biguint(BigUint::new(hm_z2)),
    ]);

    let sx_fp2 = Fp2([
        Fp::get_fp_from_biguint(BigUint::new(s_x1)),
        Fp::get_fp_from_biguint(BigUint::new(s_x2)),
    ]);
    let sy_fp2 = Fp2([
        Fp::get_fp_from_biguint(BigUint::new(s_y1)),
        Fp::get_fp_from_biguint(BigUint::new(s_y2)),
    ]);
    let sz_fp2 = Fp2([
        Fp::get_fp_from_biguint(BigUint::new(s_z1)),
        Fp::get_fp_from_biguint(BigUint::new(s_z2)),
    ]);

    let g_x = Fp::get_fp_from_biguint(BigUint::new(gx));
    let g_y = Fp::get_fp_from_biguint(BigUint::new(gy));
    // 2. P(pk_negate, Hm)
    let e_p_hm = pairing(pk_x_neg_fp, pk_y_neg_fp, hmx_fp2, hmy_fp2, hmz_fp2);
    let e_g_s = pairing(g_x, g_y, sx_fp2, sy_fp2, sz_fp2);

    let mu = e_p_hm * e_g_s;

    let mu_finaexp = mu.final_exponentiate();

    mu_finaexp == Fp12::one()
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use num_bigint::BigUint;
    use crate::verification::utils::native_bls::sub_u32_slices_12;

    use super::{get_u32_vec_from_literal, modulus, verify_bls_signatures, Fp12};

    #[test]
    pub fn test_bls_signature_verification() {
        assert!(verify_bls_signatures());
    }

    #[test]
    pub fn test_final_exponentiate() {
        let aa = ["2181142506194812233868097821779361009807326315828153071050324314717744521676711650071190927260282422014627435089208",
            "3266212670671256779826008414922395966600400122723332695666308996296105595418386213353825620535446475769829785237189",
            "3280330655787598118299804758957910379684134784964426565939861302675766948066521588562898980898245868682162153155911",
            "333668007718210311816046938245689395232794221928183840372182128979685996722059498232053963662509478803385469716056",
            "1650925102445293819378017648160637800280351377141029658990698964033732511884552459036333864590686008335846481856882",
            "3925133212240632255860280854235945320282874550806663137653784505923891479863770370026712801361887427462376126696706",
            "2444089052091192833501409081021321360112867893942837175254954622703299880931587618210267154453853513743076365662283",
            "3142914221549818039420055870398197863502329018278548609868118001898418737390067291084903575823960349378631910285921",
            "1952057563719092278028425573632201081234877258097927010867141683896274170520489868686437644804596724295624637397077",
            "254131389529427774765960554324483250584297364987873642087841623909520980093766889928789173976296059957431962608694",
            "1385128161651935856764061834929068245137081648283968377947672499160305921464670953157912428887005620142387465559867",
            "101302147352745188522496764263445345397483945567997375025250825330209385517139484882425580831299520200841767383756"];

        let aa_fp12 = Fp12::from_str(aa);
        let mu_finaexp = aa_fp12.final_exponentiate();
        mu_finaexp.print();
        assert_eq!(mu_finaexp, Fp12::one())
    }

    #[test]
    fn test_subu32() {
        let x: BigUint = BigUint::from_str("1").unwrap() << 381;
        let y = modulus();
        let x_u32 = get_u32_vec_from_literal(x.clone());
        let y_u32 = get_u32_vec_from_literal(y.clone());
        let (res, _carries) = sub_u32_slices_12(&x_u32, &y_u32);
        assert_eq!(x - y, BigUint::new(res.to_vec()));
    }
}
