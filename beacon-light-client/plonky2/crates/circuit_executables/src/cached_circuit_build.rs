use std::{fs, marker::PhantomData};

use circuit::{Circuit, CircuitTargetType, SerdeCircuitTarget};
use plonky2::{
    field::extension::Extendable,
    hash::hash_types::RichField,
    plonk::{
        circuit_data::CircuitData,
        config::{AlgebraicHasher, GenericConfig},
    },
    util::serialization::{Buffer, IoResult},
};
use plonky2_circuit_serializer::serializer::{CustomGateSerializer, CustomGeneratorSerializer};

pub const SERIALIZED_CIRCUITS_DIR: &str = "serialized_circuits";

pub struct CircuitTargetAndData<T: Circuit> {
    pub target: CircuitTargetType<T>,
    pub data: CircuitData<T::F, T::C, 2>,
}

pub fn get_serialized_circuit_data_path(circuit_name: &str, level: usize) -> String {
    format!("{SERIALIZED_CIRCUITS_DIR}/{circuit_name}_{level}.plonky2_circuit")
}

pub fn get_serialized_circuit_target_path(circuit_name: &str, level: usize) -> String {
    format!("{SERIALIZED_CIRCUITS_DIR}/{circuit_name}_{level}.plonky2_targets")
}

pub fn serialize_recursive_circuit_single_level<
    T: SerdeCircuitTarget,
    F: RichField + Extendable<D>,
    C: GenericConfig<D, F = F> + 'static,
    const D: usize,
>(
    target: &T,
    circuit_data: &CircuitData<F, C, D>,
    circuit_name: &str,
    level: usize,
) where
    <C as GenericConfig<D>>::Hasher: AlgebraicHasher<F>,
{
    fs::create_dir_all(SERIALIZED_CIRCUITS_DIR).unwrap();

    let data_bytes = circuit_data
        .to_bytes(
            &CustomGateSerializer,
            &CustomGeneratorSerializer {
                _phantom: PhantomData::<C>,
            },
        )
        .unwrap();

    fs::write(
        &format!("{SERIALIZED_CIRCUITS_DIR}/{circuit_name}_{level}.plonky2_circuit",),
        &data_bytes,
    )
    .unwrap();

    let target_bytes = target.serialize().unwrap();

    fs::write(
        &format!("{SERIALIZED_CIRCUITS_DIR}/{circuit_name}_{level}.plonky2_targets",),
        &target_bytes,
    )
    .unwrap();
}

pub fn deserialize_recursive_circuit_single_level<
    T: SerdeCircuitTarget,
    F: RichField + Extendable<D>,
    C: GenericConfig<D, F = F> + 'static,
    const D: usize,
>(
    circuit_name: &str,
    level: usize,
) -> IoResult<(T, CircuitData<F, C, D>)>
where
    <C as GenericConfig<D>>::Hasher: AlgebraicHasher<F>,
{
    let circuit_data_bytes =
        fs::read(get_serialized_circuit_data_path(circuit_name, level)).unwrap();

    let circuit_target_bytes =
        fs::read(get_serialized_circuit_target_path(circuit_name, level)).unwrap();

    let circuit_data = CircuitData::<F, C, D>::from_bytes(
        &circuit_data_bytes,
        &CustomGateSerializer,
        &CustomGeneratorSerializer {
            _phantom: PhantomData::<C>,
        },
    )?;

    let mut buffer = Buffer::new(&circuit_target_bytes);
    let circuit_target = T::deserialize(&mut buffer)?;

    Ok((circuit_target, circuit_data))
}

pub fn build_recursive_circuit_single_level_cached<
    T: SerdeCircuitTarget,
    F: RichField + Extendable<D>,
    C: GenericConfig<D, F = F> + 'static,
    const D: usize,
>(
    circuit_name: &str,
    level: usize,
    circuit_build_proc: &impl Fn() -> (T, CircuitData<F, C, D>),
) -> (T, CircuitData<F, C, D>)
where
    <C as GenericConfig<D>>::Hasher: AlgebraicHasher<F>,
{
    let data_exists = path_exists(&get_serialized_circuit_data_path(circuit_name, level));
    let target_exists = path_exists(&get_serialized_circuit_target_path(circuit_name, level));

    if !data_exists || !target_exists {
        let (target, data) = circuit_build_proc();
        serialize_recursive_circuit_single_level(&target, &data, circuit_name, level);
        (target, data)
    } else {
        deserialize_recursive_circuit_single_level(circuit_name, level).unwrap()
    }
}

pub fn build_recursive_circuit_cached<FC: Circuit, IC: Circuit<F = FC::F, C = FC::C>>(
    circuit_name: &str,
    depth: usize,
    build_first_level_proc: &impl Fn() -> (FC::Target, CircuitData<FC::F, FC::C, 2>),
    build_inner_level_proc: &impl Fn(
        &CircuitData<IC::F, IC::C, 2>,
    ) -> (IC::Target, CircuitData<IC::F, IC::C, 2>),
) -> (CircuitTargetAndData<FC>, Vec<CircuitTargetAndData<IC>>)
where
    <FC as Circuit>::C: 'static,
    <FC as Circuit>::Target: SerdeCircuitTarget,
    <IC as Circuit>::Target: SerdeCircuitTarget,
    <<FC as Circuit>::C as GenericConfig<2>>::Hasher: AlgebraicHasher<<FC as Circuit>::F>,
{
    let (first_level_target, first_level_data) =
        build_recursive_circuit_single_level_cached(circuit_name, 0, build_first_level_proc);

    let first_level_circuit = CircuitTargetAndData::<FC> {
        target: first_level_target,
        data: first_level_data,
    };

    let mut inner_level_circuits = vec![];
    let mut prev_circuit_data = &first_level_circuit.data;

    for level in 1..=depth {
        let (target, data) =
            build_recursive_circuit_single_level_cached(circuit_name, level, &|| {
                build_inner_level_proc(prev_circuit_data)
            });

        inner_level_circuits.push(CircuitTargetAndData::<IC> { target, data });
        prev_circuit_data = &inner_level_circuits.last().unwrap().data;
    }

    (first_level_circuit, inner_level_circuits)
}

fn path_exists(path: &str) -> bool {
    fs::metadata(path).is_ok()
}
