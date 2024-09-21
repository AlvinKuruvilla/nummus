use ark_ff::PrimeField;
use ark_r1cs_std::{alloc::AllocVar, fields::fp::FpVar};
use ark_relations::r1cs::ConstraintSystem;
use dusk_bls12_381::BlsScalar;
use dusk_poseidon::{Domain, Hash};

use crate::{gadgets::merkle_gadget::MerkleTreeGadget, utils::fpvar_to_u64};
fn convert_scalars_to_single_u32(scalars: Vec<BlsScalar>) -> u32 {
    let mut total: u64 = 0;

    for scalar in scalars {
        // Get the internal representation as &[u64; 4]
        let repr = scalar.internal_repr();
        // Combine the first u64 for this example (you could adjust this logic)
        total = total.wrapping_add(repr[0]);
    }

    total as u32 // Truncate to u32
}

/// The goal here is take the merkle tree roots and recreate the Merkle tree root. From there, we can apply a fiat-shamir style
/// transform to it by hashing it with the Poseidon hash. This hashed result is then the alpha we can use in the asset proof.
pub fn generate_alpha<F: PrimeField>(blockchain_merkle_roots: Vec<F>) -> u32 {
    let cs = ConstraintSystem::<F>::new_ref();
    let merkle_roots: Vec<FpVar<F>> = blockchain_merkle_roots
        .into_iter()
        .map(|field_element| {
            FpVar::new_input(cs.clone(), || Ok(field_element)).expect("Failed to create FpVar")
        })
        .collect();

    let cs = ConstraintSystem::<F>::new_ref();
    for (idx, _) in merkle_roots.iter().enumerate() {
        // Verify the chosen leaf
        let is_valid =
            MerkleTreeGadget::generate_proof_and_validate(&merkle_roots, cs.clone(), vec![idx]);
        if !is_valid {
            panic!("Cannot get root hash if leaves are not all valid");
        }
    }
    let root = MerkleTreeGadget::create_root_hash(merkle_roots, cs);
    let base = fpvar_to_u64(&root).unwrap();
    let input = BlsScalar::from_raw([base, base, base, base]);
    let hash = Hash::digest(Domain::Other, &[input; 1]);
    convert_scalars_to_single_u32(hash)
}
