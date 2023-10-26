use crate::utils::{
    universal::{assert_is_true, div_rem, div_rem_u64, exp_from_bits, le_sum},
    variable::to_bits,
};
use itertools::Itertools;
use plonky2::field::types::Field;
use plonky2x::{
    backend::circuit::Circuit,
    prelude::{
        BoolVariable, ByteVariable, Bytes32Variable, BytesVariable, CircuitBuilder,
        CircuitVariable, PlonkParameters, U64Variable, Variable,
    },
};

fn to_bin<L: PlonkParameters<D>, const D: usize>(
    builder: &mut CircuitBuilder<L, D>,
    hashed: Bytes32Variable,
    start_idx: usize,
    end_idx: usize,
) -> Variable {
    let mut hashed_half: Vec<BoolVariable> = Vec::new();
    for i in start_idx..end_idx {
        for j in 0..8 {
            hashed_half.push(hashed.0 .0[7 - i].0[j]);
        }
    }

    let const_2: Variable = builder.constant(L::Field::from_canonical_usize(2));

    let mut pow_2 = builder.constant(L::Field::from_canonical_usize(1));
    let mut bin = builder.constant(L::Field::from_canonical_usize(0));
    for i in 0..32 {
        let addend = builder.mul(hashed_half[31 - i].0, pow_2);
        bin = builder.add(addend, bin);
        pow_2 = builder.mul(const_2, pow_2);
    }

    bin
}

#[derive(Debug, Clone)]
struct ComputeShuffledIndex;

impl Circuit for ComputeShuffledIndex {
    fn define<L: PlonkParameters<D>, const D: usize>(builder: &mut CircuitBuilder<L, D>) {
        let mut index = builder.read::<U64Variable>();
        let index_count = builder.read::<U64Variable>();
        let seed = builder.read::<Bytes32Variable>();

        let index_lte_index_count = builder.lte(index, index_count);
        assert_is_true(builder, index_lte_index_count); // Check if that's true

        let const_0: Variable = builder.constant(L::Field::from_canonical_usize(0));
        let const_2: Variable = builder.constant(L::Field::from_canonical_usize(2));
        let const_1 = builder.constant::<U64Variable>(2);
        let const_2u64 = builder.constant::<U64Variable>(2);
        let const_8 = builder.constant::<U64Variable>(8);
        let const_256 = builder.constant::<U64Variable>(256);
        let const_0_byte: ByteVariable = ByteVariable::constant(builder, 0);
        const SHUFFLE_ROUND_COUNT: usize = 10;
        for current_round in 0..SHUFFLE_ROUND_COUNT {
            let current_round_bytes: ByteVariable =
                ByteVariable::constant(builder, current_round as u8);

            let mut seed_round_to_be_hashed: BytesVariable<33> = BytesVariable([const_0_byte; 33]);
            for i in 0..32 {
                seed_round_to_be_hashed.0[i] = seed.0 .0[i];
            }
            seed_round_to_be_hashed.0[32] = current_round_bytes;

            let seed_current_round_hashed: Bytes32Variable =
                builder.sha256(&seed_round_to_be_hashed.0);
            let seed_current_round_hashed_1: Vec<[BoolVariable; 8]> = seed_current_round_hashed
                .as_bytes()
                .iter()
                .map(|x| x.as_be_bits())
                .collect();
            let seed_current_round_hashed_2: Bytes32Variable = Bytes32Variable(BytesVariable(
                seed_current_round_hashed_1
                    .iter()
                    .map(|x| ByteVariable(*x))
                    .collect::<Vec<_>>()
                    .try_into()
                    .unwrap(),
            ));

            let first_seed_current_round_hash_to_variable =
                to_bin(builder, seed_current_round_hashed_2, 0, 4);
            let second_seed_current_round_hash_to_variable =
                to_bin(builder, seed_current_round_hashed_2, 4, 8);

            let seed_current_round_hash_to_u64variable_combined = U64Variable::from_variables(
                builder,
                &[
                    second_seed_current_round_hash_to_variable,
                    first_seed_current_round_hash_to_variable,
                ],
            );

            let pivot = div_rem_u64(
                builder,
                seed_current_round_hash_to_u64variable_combined,
                index_count,
            );

            // builder.watch(&pivot, format!("pivot [{}]:", current_round).as_str());

            let sum_pivot_index_count = builder.add(pivot, index_count);
            let sum_pivot_index_count_sub_index = builder.sub(sum_pivot_index_count, index);
            let flip = div_rem_u64(builder, sum_pivot_index_count_sub_index, index_count);
            // builder.watch(&flip, format!("flip [{}]:", current_round).as_str());

            let index_lte_flip = builder.lte(index, flip);
            let position = builder.select(index_lte_flip, flip, index);

            // builder.watch(&position, format!("position [{}]:", current_round).as_str());

            // builder.watch(&seed, format!("seed [{}]:", current_round).as_str());

            let position_div_256 = builder.div(position, const_256);
            // builder.watch(
            //     &position_div_256,
            //     format!("position_div_256 {}", current_round).as_str(),
            // );
            let position_div_256_bits = builder.to_le_bits(position_div_256);
            // for i in 0..position_div_256_bits.len() {
            //     builder.watch(
            //         &position_div_256_bits[i].0,
            //         format!("position_div_256_bits[{}] {}", i, current_round).as_str(),
            //     );
            // }
            // let y = le_sum(builder, &position_div_256_bits);
            let mut position_div_256_bytes = Vec::new();

            for i in 0..4 {
                let bits = position_div_256_bits[(i * 8)..((i + 1) * 8)]
                    .iter()
                    .rev()
                    .map(|x| x.0)
                    .collect_vec();
                position_div_256_bytes.push(ByteVariable::from_variables(builder, bits.as_slice()));
            }
            // for i in 0..position_div_256_bytes.len() {
            //     builder.watch(
            //         &position_div_256_bytes[i],
            //         format!("position_div_256_bytes[{}] {}", i, current_round).as_str(),
            //     );
            // }

            // for i in 0..position_div_256_bits.len() {
            //     builder.watch(
            //         &position_div_256_bits[i].0,
            //         format!("pos 256 byte [{}] {}:", i, current_round).as_str(),
            //     );
            // }

            let mut source_to_be_hashed: BytesVariable<37> = BytesVariable([const_0_byte; 37]);
            for i in 0..32 {
                source_to_be_hashed.0[i] = seed.0 .0[i];
            }
            source_to_be_hashed.0[32] = current_round_bytes;
            for i in 0..4 {
                source_to_be_hashed.0[33 + i] = position_div_256_bytes[i];
            }

            let source = builder.sha256(&source_to_be_hashed.0);
            // builder.watch(&source, format!("source {}", current_round).as_str());

            let mut test = Vec::new();
            for i in 0..32 {
                for j in 0..8 {
                    test.push(source.0 .0[i].0[j].0);
                }
            }

            // debug::debug(
            //     builder,
            //     test.iter().map(|_| format!("source")).collect_vec(),
            //     test,
            // );

            let position_mod_256 = div_rem_u64(builder, position, const_256);
            let position_mod_256_div_8 = builder.div(position_mod_256, const_8);

            // builder.watch(
            //     &position_mod_256,
            //     format!("position_mod_256 {}", current_round).as_str(),
            // );
            // builder.watch(
            //     &position_mod_256_div_8,
            //     format!("position_mod_256_div_8 {}", current_round).as_str(),
            // );

            let position_mod_256_div_8_bits = builder.to_le_bits(position_mod_256_div_8);
            // for i in 0..position_mod_256_div_8_bits.len() {
            //     builder.watch(&position_mod_256_div_8_bits[i].0, "position_mod_256_div_8_bits index: ");
            // }

            let mut power_of_2 = builder.constant(L::Field::ONE);
            let mut position_mod_256_div_8_variable = builder.constant(L::Field::ZERO);
            for i in 0..64 {
                let addend = builder.mul(position_mod_256_div_8_bits[i].0, power_of_2);
                position_mod_256_div_8_variable =
                    builder.add(addend, position_mod_256_div_8_variable);
                power_of_2 = builder.mul(const_2, power_of_2);
            }

            // builder.watch(
            //     &position_mod_256_div_8_variable,
            //     format!("position_mod_256_div_8_variable {}", current_round).as_str(),
            // );

            let byte = builder.select_array(&source.0 .0, position_mod_256_div_8_variable);
            let byte_to_variable = byte.to_variable(builder);
            // builder.watch(
            //     &byte_to_variable,
            //     format!("---> byte_to_variable {}", current_round).as_str(),
            // );

            let position_mod_8 = div_rem_u64(builder, position, const_8);
            // builder.watch(
            //     &position_mod_8,
            //     format!("position_mod_8 {}", current_round).as_str(),
            // );

            let position_mod_8_to_bits = builder.to_le_bits(position_mod_8);
            // for i in 0..position_mod_8_to_bits.len() {
            //     builder.watch(&position_mod_8_to_bits[i].0, "position_mod_8_to_bits");
            // }
            let const_2_pow_position_mod_8 =
                exp_from_bits(builder, const_2, &position_mod_8_to_bits);
            // builder.watch(
            //     &const_2_pow_position_mod_8,
            //     format!("const_2_pow_position_mod_8 {}", current_round).as_str(),
            // );

            let byte_u_var = U64Variable::from_variables(builder, &[byte_to_variable, const_0]);
            // builder.watch(
            //     &byte_u_var,
            //     format!("byte_u_var {}", current_round).as_str(),
            // );

            let const_2_pow_position_mod_8_u =
                U64Variable::from_variables(builder, &[const_2_pow_position_mod_8, const_0]);

            let byte_shl_position_mod_8 = builder.div(byte_u_var, const_2_pow_position_mod_8_u);
            // builder.watch(
            //     &byte_shl_position_mod_8,
            //     format!("byte_shl_position_mod_8 {}", current_round).as_str(),
            // );
            // builder.watch(&byte_shl_position_mod_8, "byte_shl_position_mod_8");

            let bit = div_rem_u64(builder, byte_shl_position_mod_8, const_2u64);
            // builder.watch(&bit, format!("bit [{}]:", current_round).as_str());
            // builder.watch(
            //     &bit.variables()[0],
            //     format!("bit vars[0] [{}]:", current_round).as_str(),
            // );
            // builder.watch(
            //     &bit.variables()[1],
            //     format!("bit vars[1] [{}]:", current_round).as_str(),
            // );

            // index = if builder.is_zero(bit) { flip } else { index };
            index = builder.select(BoolVariable(bit.variables()[0]), flip, index);
        }

        //builder.watch(&index, "index");
        builder.write::<U64Variable>(index);
    }
}

#[cfg(test)]
mod tests {
    use plonky2x::utils;

    use super::*;
    use plonky2x::backend::circuit::DefaultParameters;
    use plonky2x::frontend::vars::{ArrayVariable, ByteVariable};
    use plonky2x::prelude::{bytes, DefaultBuilder};

    #[test]
    fn test_compute_shuffled_index_0() {
        utils::setup_logger();

        let seed_bytes: Vec<u8> =
            bytes!("0x23bcd11624a07465b1c2fc1a0fe52996daae4bf87b0fb6bed45926096c644843");

        let mut seed_bytes_fixed_size = [0u8; 32];
        seed_bytes_fixed_size[..32].copy_from_slice(&seed_bytes);

        let mut builder = CircuitBuilder::<DefaultParameters, 2>::new();
        ComputeShuffledIndex::define(&mut builder);

        let circuit = builder.build();

        const start_idx: u64 = 0;
        const count: u64 = 1000;
        let mapping = [820, 167, 701, 617, 678, 127, 640, 956, 940, 37, 853, 194, 348, 119, 347,
        453, 760, 57, 770, 80, 288, 58, 537, 460, 248, 187, 276, 536, 855, 544, 250, 415,
        212, 285, 754, 188, 129, 830, 76, 457, 252, 561, 902, 838, 615, 85, 47, 223, 778,
        84, 787, 303, 630, 328, 2, 722, 530, 631, 578, 857, 293, 432, 501, 649, 409, 825,
        142, 873, 413, 622, 858, 137, 609, 355, 97, 34, 613, 614, 612, 834, 784, 286, 846,
        164, 591, 836, 442, 824, 161, 542, 994, 397, 213, 738, 556, 259, 388, 191, 708,
        551, 333, 117, 739, 96, 217, 527, 957, 607, 827, 727, 436, 105, 611, 756, 63, 208,
        908, 592, 95, 748, 892, 18, 833, 683, 786, 952, 67, 763, 582, 386, 533, 911, 980,
        888, 379, 295, 514, 811, 206, 513, 534, 819, 109, 174, 997, 42, 996, 380, 199, 897,
        168, 943, 960, 668, 659, 71, 931, 942, 832, 425, 287, 977, 222, 77, 198, 666, 406,
        91, 113, 503, 886, 519, 518, 945, 949, 843, 487, 806, 140, 4, 378, 193, 866, 404,
        891, 458, 693, 197, 923, 74, 356, 644, 150, 593, 705, 887, 443, 240, 156, 421, 606,
        363, 884, 410, 49, 474, 663, 681, 654, 863, 516, 383, 826, 740, 262, 427, 620, 828,
        950, 272, 531, 509, 961, 647, 469, 237, 579, 804, 528, 243, 316, 6, 714, 284, 491,
        768, 505, 674, 1, 984, 946, 702, 914, 353, 676, 382, 697, 634, 48, 540, 618, 475,
        621, 869, 999, 485, 488, 313, 255, 357, 10, 982, 318, 190, 323, 662, 599, 896, 526,
        239, 364, 29, 650, 791, 893, 511, 742, 135, 753, 307, 38, 758, 587, 311, 944, 780,
        550, 260, 854, 822, 875, 468, 120, 324, 510, 130, 327, 370, 713, 495, 915, 218,
        494, 21, 741, 794, 124, 926, 154, 98, 435, 679, 440, 696, 685, 736, 8, 965, 759,
        545, 350, 152, 948, 39, 573, 655, 574, 59, 270, 234, 671, 694, 577, 151, 465, 835,
        148, 554, 496, 374, 497, 314, 352, 799, 309, 608, 687, 912, 992, 219, 90, 101, 266,
        83, 907, 36, 424, 162, 100, 523, 499, 305, 28, 22, 258, 267, 664, 823, 560, 462,
        669, 46, 935, 317, 909, 851, 745, 35, 726, 874, 789, 112, 969, 506, 848, 275, 645,
        571, 247, 716, 629, 504, 261, 103, 688, 959, 32, 689, 522, 396, 16, 27, 895, 771,
        782, 89, 368, 185, 575, 792, 78, 583, 548, 628, 473, 381, 196, 51, 138, 517, 788,
        310, 856, 486, 482, 337, 625, 932, 371, 492, 894, 643, 972, 549, 829, 602, 20, 452,
        254, 670, 450, 268, 632, 938, 885, 535, 173, 985, 419, 555, 490, 847, 619, 346,
        842, 627, 231, 467, 801, 132, 813, 939, 970, 648, 744, 718, 478, 339, 417, 395,
        201, 849, 17, 646, 711, 515, 229, 429, 102, 362, 163, 658, 94, 880, 92, 256, 204,
        9, 698, 774, 202, 539, 929, 601, 215, 44, 841, 567, 785, 746, 868, 604, 808, 747,
        407, 816, 241, 761, 470, 749, 390, 521, 391, 360, 111, 899, 449, 520, 584, 319,
        438, 916, 717, 703, 910, 159, 143, 989, 967, 962, 570, 416, 279, 953, 557, 968,
        14, 781, 817, 955, 301, 553, 399, 598, 289, 123, 987, 312, 221, 246, 472, 565, 610,
        344, 414, 175, 983, 974, 973, 64, 653, 723, 730, 883, 765, 72, 541, 920, 141, 118,
        512, 437, 5, 889, 245, 752, 642, 345, 675, 43, 797, 251, 282, 277, 210, 862, 200,
        361, 54, 737, 439, 657, 777, 375, 195, 296, 821, 13, 775, 837, 115, 340, 33, 24,
        116, 586, 308, 299, 320, 445, 922, 170, 230, 209, 680, 420, 546, 850, 900, 750,
        852, 919, 392, 257, 430, 904, 394, 901, 898, 326, 306, 400, 68, 176, 351, 330, 216,
        755, 979, 298, 928, 860, 122, 441, 596, 562, 660, 139, 707, 566, 917, 302, 934,
        633, 41, 86, 464, 131, 65, 795, 558, 297, 79, 710, 134, 104, 81, 715, 954, 975,
        12, 605, 635, 448, 249, 493, 572, 936, 743, 238, 366, 332, 877, 235, 684, 814, 73,
        271, 236, 82, 56, 50, 377, 543, 211, 867, 971, 728, 483, 993, 133, 471, 879, 454,
        126, 315, 677, 372, 128, 385, 538, 951, 157, 228, 489, 990, 595, 93, 844, 809, 508,
        207, 461, 384, 767, 652, 418, 403, 498, 590, 692, 3, 331, 861, 55, 547, 31, 153,
        568, 373, 281, 273, 998, 177, 23, 569, 149, 802, 60, 75, 840, 160, 367, 859, 11,
        189, 988, 456, 15, 941, 205, 695, 890, 220, 477, 552, 735, 691, 99, 431, 927, 169,
        845, 581, 145, 793, 720, 921, 53, 411, 376, 638, 783, 764, 479, 815, 690, 597, 563,
        269, 7, 976, 800, 933, 226, 155, 114, 766, 624, 872, 263, 62, 108, 484, 203, 995,
        365, 244, 731, 724, 637, 925, 158, 559, 401, 125, 665, 818, 769, 300, 734, 790,
        585, 447, 864, 641, 798, 455, 839, 121, 757, 66, 706, 594, 280, 656, 876, 947, 986,
        905, 870, 700, 106, 25, 937, 981, 651, 40, 733, 623, 751, 192, 338, 325, 500, 529,
        672, 402, 963, 930, 721, 958, 336, 446, 434, 166, 30, 242, 214, 292, 387, 725, 405,
        423, 667, 358, 466, 686, 812, 283, 19, 70, 278, 181, 61, 580, 342, 600, 426, 359,
        807, 136, 991, 588, 0, 913, 52, 796, 805, 682, 182, 186, 576, 393, 463, 865, 776,
        172, 964, 709, 291, 274, 882, 354, 978, 673, 369, 178, 322, 661, 253, 144, 603,
        26, 831, 179, 732, 349, 183, 227, 334, 966, 636, 147, 184, 502, 564, 871, 107, 918,
        88, 171, 626, 924, 878, 433, 224, 762, 87, 412, 232, 881, 265, 165, 481, 507, 389,
        428, 480, 476, 589, 398, 532, 290, 524, 639, 146, 906, 45, 719, 422, 180, 225, 343,
        335, 773, 903, 451, 699, 810, 321, 341, 264, 704, 459, 803, 729, 110, 304, 294,
        779, 69, 444, 329, 233, 772, 525, 712, 616, 408];
        for i in start_idx..count {
            let mut input = circuit.input();

            input.write::<U64Variable>(i);
            input.write::<U64Variable>(count);
            input.write::<ArrayVariable<ByteVariable, 32>>(seed_bytes_fixed_size.to_vec());

            let (_witness, mut _output) = circuit.prove(&input);
            circuit.verify(&_witness, &input, &_output);
            let shuffled_index_res = _output.read::<U64Variable>();

            println!("{} {}", mapping[i as usize], shuffled_index_res);
            assert!(mapping[i as usize] == shuffled_index_res);
        }
    }

    // #[test]
    // fn test_compute_shuffled_index_10() {
    //     utils::setup_logger();
    //     let mut builder: CircuitBuilder<DefaultParameters, 2> = DefaultBuilder::new();

    //     let index = builder.read::<U64Variable>();
    //     let index_count = builder.read::<U64Variable>();
    //     let seed = builder.read::<Bytes32Variable>();

    //     let shuffled_index = compute_shuffled_index(&mut builder, index, index_count, seed);

    //     builder.watch(&shuffled_index, "shuffled_index");
    //     let circuit = builder.mock_build();

    //     let mut input = circuit.input();
    //     let seed_bytes: Vec<u8> =
    //         bytes!("0x2c7c329908222b0e98b0dc09c8e92c6f28b2abb4c6b5300f4244e6b740311f88");

    //     let mut seed_bytes_fixed_size = [0u8; 32];
    //     seed_bytes_fixed_size[..32].copy_from_slice(&seed_bytes);

    //     let mapping = [2, 5, 6, 0, 1, 7, 4, 3, 8, 9];
    //     let mut res: [u64; 10] = [0; 10];
    //     for i in 0..10 {
    //         input.write::<U64Variable>(i);
    //         input.write::<U64Variable>(10);
    //         input.write::<ArrayVariable<ByteVariable, 32>>(seed_bytes_fixed_size.to_vec());

    //         let (_witness, mut _output) = circuit.mock_prove(&input);
    //         let shuffled_index_res = shuffled_index.get(&_witness);
    //         res[i as usize] = shuffled_index_res;
    //     }

    //     for i in 0..10 {
    //         assert!(mapping[i] == res[i], "Fails at {} index", i);
    //         println!("{} {}", mapping[i as usize], res[i as usize]);
    //     }
    // }
}
