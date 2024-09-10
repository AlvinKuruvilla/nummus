use ark_bn254::Bn254;
use ark_ff::Field;
use ark_groth16::{r1cs_to_qap::LibsnarkReduction, Groth16, Proof, ProvingKey};
use ark_relations::r1cs::{ConstraintSynthesizer, ConstraintSystemRef, SynthesisError};
use rand::rngs::OsRng;

use crate::ducat_types::org::{validate_transaction_roots, validate_transaction_serial_numbers};

pub struct BlockchainValidatorCircuit {
    pub blockchain_sns: Vec<u64>,         // serial numbers
    pub blockchain_roots: Vec<u64>,       // root hashes
    pub transaction_root_cache: Vec<u64>, // transaction root cache
    pub spent_serial_numbers: Vec<u64>,   // spent serial numbers
}
impl BlockchainValidatorCircuit {
    pub fn new(
        blockchain_sns: Vec<u64>,
        blockchain_roots: Vec<u64>,
        transaction_root_cache: Vec<u64>,
        spent_serial_numbers: Vec<u64>,
    ) -> Self {
        Self {
            blockchain_sns,
            blockchain_roots,
            transaction_root_cache,
            spent_serial_numbers,
        }
    }
}
impl<F: Field> ConstraintSynthesizer<F> for BlockchainValidatorCircuit {
    fn generate_constraints(self, cs: ConstraintSystemRef<F>) -> Result<(), SynthesisError> {
        if !validate_transaction_serial_numbers(self.blockchain_sns, self.spent_serial_numbers) {
            panic!("Could not validate spent transaction serial numbers");
        }
        if !validate_transaction_roots(self.blockchain_roots, self.transaction_root_cache) {
            panic!("Could not validate spent transaction Merkle tree root");
        }
        Ok(())
    }
}
pub fn blockchain_validator_generate_proof(
    blockchain_sns: Vec<u64>,
    blockchain_roots: Vec<u64>,
    transaction_root_cache: Vec<u64>,
    spent_serial_numbers: Vec<u64>,
    proving_key: &ProvingKey<Bn254>,
) -> Proof<Bn254> {
    let circuit = BlockchainValidatorCircuit::new(
        blockchain_sns,
        blockchain_roots,
        transaction_root_cache,
        spent_serial_numbers,
    );

    // Generate the proof using Groth16 with the LibsnarkReduction
    let rng = &mut OsRng;
    Groth16::<Bn254, LibsnarkReduction>::create_random_proof_with_reduction(
        circuit,
        proving_key,
        rng,
    )
    .unwrap()
}
