use plonky2::{
    field::{goldilocks_field::GoldilocksField, types::Field},
    hash::{hashing::hash_n_to_hash_no_pad, poseidon::{self, PoseidonHash, PoseidonPermutation}},
};

pub fn do_something() {
    println!("Did something !");
    let r = hash_n_to_hash_no_pad::<GoldilocksField, PoseidonPermutation<GoldilocksField>>(&[GoldilocksField::from_canonical_u64(123)]);

    println!("r = {:?}", r)
}
