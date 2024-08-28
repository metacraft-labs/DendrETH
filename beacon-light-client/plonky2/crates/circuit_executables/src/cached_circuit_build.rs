use anyhow::Result;
use std::{fs, marker::PhantomData};

use circuit::{
    serde_circuit_target::deserialize_circuit_target, Circuit, CircuitTargetType,
    SerdeCircuitTarget,
};
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

use crate::crud::common::read_from_file;

fn load_circuit_target_recursive<T: Circuit>(
    dir: &str,
    circuit_name: &str,
    level: usize,
) -> Result<CircuitTargetType<T>>
where
    <T as Circuit>::Target: SerdeCircuitTarget,
{
    let target_bytes = read_from_file(&format!("{dir}/{circuit_name}_{level}.plonky2_targets"))?;
    let mut target_buffer = Buffer::new(&target_bytes);

    Ok(deserialize_circuit_target::<T>(&mut target_buffer).unwrap())
}

pub fn load_circuit_data_recursive<T: Circuit>(
    dir: &str,
    circuit_name: &str,
    level: usize,
) -> Result<CircuitData<T::F, T::C, 2>>
where
    <<T as Circuit>::C as GenericConfig<2>>::Hasher: AlgebraicHasher<<T as Circuit>::F>,
    <T as Circuit>::C: 'static,
{
    let gate_serializer = CustomGateSerializer;
    let generator_serializer = CustomGeneratorSerializer {
        _phantom: PhantomData::<T::C>,
    };

    let circuit_data_bytes =
        read_from_file(&format!("{dir}/{circuit_name}_{level}.plonky2_circuit"))?;

    Ok(CircuitData::<T::F, T::C, 2>::from_bytes(
        &circuit_data_bytes,
        &gate_serializer,
        &generator_serializer,
    )
    .unwrap())
}

pub struct CircuitTargetAndData<T: Circuit> {
    pub target: CircuitTargetType<T>,
    pub data: CircuitData<T::F, T::C, 2>,
}

impl<T: Circuit> CircuitTargetAndData<T> {
    pub fn load_recursive(dir: &str, name: &str, level: usize) -> Result<Self>
    where
        <T as Circuit>::Target: SerdeCircuitTarget,
        <<T as Circuit>::C as GenericConfig<2>>::Hasher: AlgebraicHasher<<T as Circuit>::F>,
        <T as Circuit>::C: 'static,
    {
        Ok(CircuitTargetAndData::<T> {
            target: load_circuit_target_recursive::<T>(dir, name, level)?,
            data: load_circuit_data_recursive::<T>(dir, name, level)?,
        })
    }
}

pub fn get_serialized_recursive_circuit_data_path(
    dir: &str,
    circuit_name: &str,
    level: usize,
) -> String {
    get_serialized_circuit_data_path(dir, format!("{circuit_name}_{level}").as_str())
}

pub fn get_serialized_recursive_circuit_target_path(
    dir: &str,
    circuit_name: &str,
    level: usize,
) -> String {
    get_serialized_circuit_target_path(dir, format!("{circuit_name}_{level}").as_str())
}

pub fn get_serialized_circuit_data_path(dir: &str, circuit_name: &str) -> String {
    format!("{dir}/{circuit_name}.plonky2_circuit")
}

pub fn get_serialized_circuit_target_path(dir: &str, circuit_name: &str) -> String {
    format!("{dir}/{circuit_name}.plonky2_targets")
}

pub fn serialize_recursive_circuit_single_level<
    T: SerdeCircuitTarget,
    F: RichField + Extendable<D>,
    C: GenericConfig<D, F = F> + 'static,
    const D: usize,
>(
    target: &T,
    circuit_data: &CircuitData<F, C, D>,
    dir: &str,
    circuit_name: &str,
    level: usize,
) where
    <C as GenericConfig<D>>::Hasher: AlgebraicHasher<F>,
{
    serialize_circuit(
        target,
        circuit_data,
        dir,
        format!("{circuit_name}_{level}").as_str(),
    )
}

pub fn deserialize_recursive_circuit_single_level<
    T: SerdeCircuitTarget,
    F: RichField + Extendable<D>,
    C: GenericConfig<D, F = F> + 'static,
    const D: usize,
>(
    dir: &str,
    circuit_name: &str,
    level: usize,
) -> IoResult<(T, CircuitData<F, C, D>)>
where
    <C as GenericConfig<D>>::Hasher: AlgebraicHasher<F>,
{
    deserialize_circuit(dir, format!("{circuit_name}_{level}").as_str())
}

pub fn serialize_circuit<
    T: SerdeCircuitTarget,
    F: RichField + Extendable<D>,
    C: GenericConfig<D, F = F> + 'static,
    const D: usize,
>(
    target: &T,
    circuit_data: &CircuitData<F, C, D>,
    dir: &str,
    circuit_name: &str,
) where
    <C as GenericConfig<D>>::Hasher: AlgebraicHasher<F>,
{
    fs::create_dir_all(dir).unwrap();

    let data_bytes = circuit_data
        .to_bytes(
            &CustomGateSerializer,
            &CustomGeneratorSerializer {
                _phantom: PhantomData::<C>,
            },
        )
        .unwrap();

    fs::write(
        &get_serialized_circuit_data_path(dir, circuit_name),
        &data_bytes,
    )
    .unwrap();

    let target_bytes = target.serialize().unwrap();

    fs::write(
        &get_serialized_circuit_target_path(dir, circuit_name),
        &target_bytes,
    )
    .unwrap();
}

pub fn deserialize_circuit<
    T: SerdeCircuitTarget,
    F: RichField + Extendable<D>,
    C: GenericConfig<D, F = F> + 'static,
    const D: usize,
>(
    dir: &str,
    circuit_name: &str,
) -> IoResult<(T, CircuitData<F, C, D>)>
where
    <C as GenericConfig<D>>::Hasher: AlgebraicHasher<F>,
{
    let circuit_data_bytes = fs::read(get_serialized_circuit_data_path(dir, circuit_name)).unwrap();

    let circuit_target_bytes =
        fs::read(get_serialized_circuit_target_path(dir, circuit_name)).unwrap();

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
    dir: &str,
    circuit_name: &str,
    level: usize,
    circuit_build_proc: &impl Fn() -> (T, CircuitData<F, C, D>),
) -> (T, CircuitData<F, C, D>)
where
    <C as GenericConfig<D>>::Hasher: AlgebraicHasher<F>,
{
    build_circuit_cached(
        dir,
        format!("{circuit_name}_{level}").as_str(),
        circuit_build_proc,
    )
}

pub fn build_circuit_cached<
    T: SerdeCircuitTarget,
    F: RichField + Extendable<D>,
    C: GenericConfig<D, F = F> + 'static,
    const D: usize,
>(
    dir: &str,
    circuit_name: &str,
    circuit_build_proc: &impl Fn() -> (T, CircuitData<F, C, D>),
) -> (T, CircuitData<F, C, D>)
where
    <C as GenericConfig<D>>::Hasher: AlgebraicHasher<F>,
{
    let data_exists = path_exists(&get_serialized_circuit_data_path(dir, circuit_name));
    let target_exists = path_exists(&get_serialized_circuit_target_path(dir, circuit_name));

    if !data_exists || !target_exists {
        let (target, data) = circuit_build_proc();
        serialize_circuit(&target, &data, dir, circuit_name);
        (target, data)
    } else {
        deserialize_circuit(dir, circuit_name).unwrap()
    }
}

pub fn build_recursive_circuit_cached<FC: Circuit, IC: Circuit<F = FC::F, C = FC::C>>(
    dir: &str,
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
        build_recursive_circuit_single_level_cached(dir, circuit_name, 0, build_first_level_proc);

    let first_level_circuit = CircuitTargetAndData::<FC> {
        target: first_level_target,
        data: first_level_data,
    };

    let mut inner_level_circuits = vec![];
    let mut prev_circuit_data = &first_level_circuit.data;

    for level in 1..=depth {
        let (target, data) =
            build_recursive_circuit_single_level_cached(dir, circuit_name, level, &|| {
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
