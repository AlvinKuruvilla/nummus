use std::collections::{HashMap, HashSet};

use ark_bn254::Bn254;
use ark_ff::Field;
use ark_groth16::{r1cs_to_qap::LibsnarkReduction, Groth16, Proof, ProvingKey};
use ark_relations::r1cs::{ConstraintSynthesizer, ConstraintSystemRef, SynthesisError};

use rand::rngs::OsRng;

use crate::core::org::validate_transaction_serial_numbers;

pub fn count_occurrences(public_vector: Vec<u64>, secret_vector: Vec<u64>) -> HashMap<u64, u32> {
    let set2: HashSet<u64> = secret_vector.iter().copied().collect();
    let mut counts = HashMap::new();

    for x in public_vector {
        if set2.contains(&x) {
            *counts.entry(x).or_insert(0) += 1;
        }
    }

    counts
}
pub fn check_general_reciprocal(num: f64, numerator: f64) -> (bool, f64) {
    let reciprocal = numerator / num;
    let result = num * reciprocal;
    ((result - numerator).abs() < f64::EPSILON, numerator) // Check if result is close to the numerator
}

pub struct AssetProof {
    alpha: u32,
    expected_occurrence_vector: Vec<u32>,
    spent_serial_numbers: Vec<u64>, // spent serial numbers
    blockchain_sns: Vec<u64>,
}
impl AssetProof {
    pub fn new(
        alpha: u32,
        expected_occurrence_vector: Vec<u32>,
        blockchain_sns: Vec<u64>,
        public_spent_serial_numbers: Vec<u64>,
    ) -> Self {
        Self {
            alpha,
            expected_occurrence_vector,
            blockchain_sns,
            spent_serial_numbers: public_spent_serial_numbers,
        }
    }
}
impl<F: Field> ConstraintSynthesizer<F> for AssetProof {
    fn generate_constraints(self, cs: ConstraintSystemRef<F>) -> Result<(), SynthesisError> {
        if !validate_transaction_serial_numbers(
            self.blockchain_sns.clone(),
            self.spent_serial_numbers.clone(),
        ) {
            panic!("Could not validate spent transaction serial numbers");
        }
        let mut actual_counts: Vec<u32> = Vec::new();
        let counts = count_occurrences(self.blockchain_sns, self.spent_serial_numbers.clone());
        let m: Vec<u32> = counts.values().copied().collect();
        let unique_public_set: HashSet<u64> = self.spent_serial_numbers.iter().copied().collect();
        let unique_public_set: Vec<u64> = Vec::from_iter(unique_public_set);
        let denominators: Vec<u64> = unique_public_set
            .iter()
            .map(|&x| x + self.alpha as u64)
            .collect();
        for (index, _) in m.clone().into_iter().enumerate() {
            let (check, _) = check_general_reciprocal(denominators[index] as f64, m[index] as f64);
            if !check {
                panic!("Reciprocals don't match the expected values");
            }
            actual_counts.push(m[index]);
        }

        // Verify that the actual counts match the expected counts
        if actual_counts != self.expected_occurrence_vector {
            panic!(
                "Expected and actual counts don't match: expected: {:?} actual: {:?}",
                self.expected_occurrence_vector, actual_counts
            );
        }
        Ok(())
    }
}
pub fn generate_asset_proof(
    alpha: u32,
    spent_serial_numbers: Vec<u64>, // spent serial numbers
    blockchain_sns: Vec<u64>,
    expected_occurrence_vector: Vec<u32>,
    proving_key: &ProvingKey<Bn254>,
) -> Proof<Bn254> {
    let circuit = AssetProof::new(
        alpha,
        expected_occurrence_vector,
        blockchain_sns,
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
