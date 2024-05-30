# Plonky2 Circuits Framework

The framework provides a common interface for all circuits. This crate, along with `circuit_derive`, automates a considerable amount of the code that one would otherwise have to write by hand in the process of writing circuits. This framework works fine on its own but is tailored to our use cases, including deserialization of the witness input and serialization of the public inputs to facilitate our distributed computation model based on Redis for IPC.

## Macro expansion

You can inspect the generated code with cargo-expand

```sh
$ cargo install cargo-expand
$ cargo expand

```

## Usage

### Circuit Target

The circuit target is a struct that contains the targets of the circuit that are either part of the witness or public inputs. The struct needs to have the `derive(CircuitTarget)` attribute. The `CircuitTarget` procedural macro provides the following helper attributes:

- `target([in|out])` - used for marking circuit target fields either as witness input (`in`) or as public input (`out`). A field can be marked as both witness input and public input with `#[target(in, out)]`. Currently there is the limitation that this attribute cannot be used on proof targets.
- `serde()` - the typical serde attribute from the serde crate. This is used to specify how the witness input and public inputs are serialized and deserialized.

```rust
// Example
#[derive(CircuitTarget)]
#[serde(rename_all = "camelCase")]
pub struct ValidatorsCommitmentMapperTarget {
    #[target(in)]
    pub validator: ValidatorTarget,

    #[target(in)]
    pub is_real: BoolTarget,

    #[target(out)]
    #[serde(with = "serde_bool_array_to_hex_string")]
    pub sha256_hash_tree_root: Sha256Target,

    #[target(out)]
    pub poseidon_hash_tree_root: HashOutTarget,
}
```

The types of the fields tagged with `target(in)` need to implement `TargetPrimitive`, `SetWitness` and `AddVirtualTarget`. All of these can be derived using the eponymous procedural macros.

The types of the fields tagged with `target(out)` need to implement `TargetPrimitive`, `PublicInputsReadable`, `PublicInputsTargetReadable` and `ToTargets`, the last 3 of which can be derived using the `PublicInputsReadable` macro.

### Associated Items

There are several associated items that need to be set when implementing the `Circuit` trait. They are the following:

- `type F: RichField + Extendable<D>`
- `type C: GenericConfig<D>` - needs to use the same field as `F`.
- `const D: usize` - this is currently always 2 since const generics are not yet mature enough but is there for completeness.
- `const CIRCUIT_CONFIG: CircuitConfig` - the config config used when building the circuit. It's left as an associated item for now but it might be supplied to the `build` function in the future.
- `type Target: ReadablePublicInputs + ReadableCircuitInputTarget` - the target struct of the circuit. The required traits are automatically derived by the `CircuitTarget` procedural macro.
- `type Params` - the type of the runtime known dependencies of the circuit. This may be used to supply circuit data in order to perform recursive proofs. In case of no dependencies this shall be set to the unit type.

### Defining the circuit

Once the associated items are in place, you can now implement the `define` method which contains the implementation of the circuit. It accepts a circuit builder and runtime-known parameters, defines the circuit through the builder and returns back the targets required for setting the witness and the ones part of the public inputs.

### Building the circuit

The circuit is built using the provided `build` method by the `Circuit` trait. it defines the circuit, registers the public inputs and builds the circuit, returning the targets and the circuit's data.

### Target Primitive

For the rest to make sense, the concept of a target primitive needs to be introduced. In order for a target to be eligible for automatic setting of the witness or reading from the public inputs, it needs to have a corresponding primitive type. The table of predefined primitives is as follows:

| Target Type     | Primitive Type                  |
| --------------- | ------------------------------- |
| `Target`        | `u64`                           |
| `BoolTarget`    | `bool`                          |
| `HashOutTarget` | `Array<u64, NUM_HASH_OUT_ELTS>` |
| `BigUintTarget` | `BigUint`                       |
| `[T; N]`        | `Array<T::Primitive, N>`        |

> **_Note:_** The `Array` type is a newtype wrapper of a Rust array that can be serialized and deserialized by `serde` for arbitrary size. This is necessary due to backwards compatibility considerations of the serde crate (currently not supporting arrays with size greater than 32).

The target primitive for product types is defined recursively and derived by the `TargetPrimitive` procedural macro. `Serialize` and `Deserialize` are derived for the primitive type. Any `serde` attributes are inherited.

```rust
// Target struct definition
#[derive(TargetPrimitive)]
#[serde(rename_all = "camelCase")]
pub struct TargetType {
    #[serde(with = "serde_bool_array_to_hex_string")]
    pub pubkey: [BoolTarget; 384],

    #[serde(with = "serde_bool_array_to_hex_string")]
    pub withdrawal_credentials: [BoolTarget; 256],

    #[serde(serialize_with = "biguint_to_str", deserialize_with = "parse_biguint")]
    pub effective_balance: BigUintTarget,

    pub slashed: BoolTarget,

   #[serde(with = "serde_bool_array_to_hex_string_nested")]
    pub merkle_branch: [[BoolTarget; 256]; 5],
}

// Generated primitive type
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TargetTypePrimitive {
    #[serde(with = "serde_bool_array_to_hex_string")]
    pub pubkey: Array<bool, 384>,

    #[serde(with = "serde_bool_array_to_hex_string")]
    pub withdrawal_credentials: Array<bool, 256>,

    #[serde(serialize_with = "biguint_to_str", deserialize_with = "parse_biguint")]
    pub effective_balance: BigUint,

    pub slashed: bool,

   #[serde(with = "serde_bool_array_to_hex_string_nested")]
    pub merkle_branch: Array<Array<bool, 256>, 5>,
}
```

### Setting the witness

The `CircuitTarget` macro derives an implementation of the `SetWitness` trait which provides a `set_witness` method on the target that sets the witness, given a `WitnessInput` struct, created from the primitive types of the target struct, marked with `target(in)`. In order for `SetWitness` to be derived, all fields marked with `in` need to implement `SetWitness`. An implementation can be derived with the `SetWitness` proc macro. These fields also need to own a primitive type since their `WitnessInput` type matches their primitive type.

```rust
// Trait definition
pub trait SetWitness<F: RichField> {
    type Input;
    fn set_witness(&self, witness: &mut PartialWitness<F>, input: &Self::Input);
}

// Usage
let mut pw = PartialWitness::new();
target.set_witness(&mut pw, &input);
```

The witness input type of a circuit can be accessed through the `CircuitInput<CircuitType>` type alias.
The private inputs can be read in a circuit using the `read_circuit_input_target` method, provided by the `Circuit` trait. Its return type can be accessed through the `CircuitInputTarget<CircuitType>` type alias.

```rust
// Circuit target struct definition
#[derive(CircuitTarget)]
#[serde(rename_all = "camelCase")]
pub struct ValidatorsCommitmentMapperTarget {
    #[target(in)]
    pub validator: ValidatorTarget,

    #[target(in)]
    pub is_real: BoolTarget,

    #[target(out)]
    #[serde(with = "serde_bool_array_to_hex_string")]
    pub sha256_hash_tree_root: Sha256Target,

    #[target(out)]
    pub poseidon_hash_tree_root: HashOutTarget,
}

// Generated WitnessInput struct
pub struct ValidatorsCommitmentMapperTargetWitnessInput {
    pub validator: <ValidatorTarget as circuit::TargetPrimitive>::Primitive,
    pub is_real: <BoolTarget as circuit::TargetPrimitive>::Primitive,
}
```

### Reading the public inputs

Reading the public inputs can be accomplished using the `Circuit` trait's `read_public_inputs` and `read_public_inputs_target` for reading a proof's public inputs outside a circuit and inside a circuit respectively. Their return types can be obtained through the `CircuitOutput<CircuitType>` and `CircuitOutputTarget<CircuitType>` type aliases.
The `CircuitOutputTarget` type contains the fields of the circuit target's fields tagged with `target(out)` and `CircuitOutput` is its primitive variant.

```rust
#[derive(CircuitTarget)]
pub struct ValidatorsCommitmentMapperTarget {
    #[target(in)]
    pub validator: ValidatorTarget,
    #[target(in)]
    pub is_real: BoolTarget,
    #[target(out)]
    #[serde(with = "serde_bool_array_to_hex_string")]
    pub sha256_hash_tree_root: Sha256Target,
    #[target(out)]
    pub poseidon_hash_tree_root: HashOutTarget,
}

pub struct ValidatorsCommitmentMapperTargetPublicInputsTarget {
    pub sha256_hash_tree_root: Sha256Target,
    pub poseidon_hash_tree_root: HashOutTarget,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ValidatorsCommitmentMapperTargetPublicInputs {
    #[serde(with = "serde_bool_array_to_hex_string")]
    pub sha256_hash_tree_root: <Sha256Target as circuit::TargetPrimitive>::Primitive,
    pub poseidon_hash_tree_root: <HashOutTarget as circuit::TargetPrimitive>::Primitive,
}
```

### Serializing/Deserializing the circuit

The serialization/deserialization logic for the circuit target is generated. An implementation can be derived through the `SerdeCircuitTarget` procedural macro. In order to be able to derive an implementation, all fields need to also implement the trait.

```rust
#[derive(CircuitTarget, SerdeCircuitTarget)]
struct ExampleCircuitTarget {
    pub target1: Target,
    pub target2: OtherTarget,
}

struct ExampleCircuit;

impl Circuit for ExampleCircuit {
    type Target = ExampleCircuitTarget;
    ...
}

// Serialization
let bytes = target.serialize().unwrap();

// Deserialization
let target = deserialize_circuit_target::<ExampleCircuit>(&mut buffer).unwrap();
```
