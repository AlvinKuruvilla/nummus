use ark_bn254::Bn254;
use ark_ff::Field;
use ark_groth16::{r1cs_to_qap::LibsnarkReduction, Groth16, Proof, ProvingKey};
use ark_relations::r1cs::{ConstraintSynthesizer, ConstraintSystemRef, SynthesisError};
use rand::rngs::OsRng;
use std::marker::PhantomData;

// Define the circuit for the zk-SNARK
pub struct EpochBalanceCircuit<F: Field> {
    pub initial_balance: i32,
    pub epoch_delta: i32,
    pub final_balance: i32,
    _marker: PhantomData<F>,
}

impl<F: Field> EpochBalanceCircuit<F> {
    pub fn new(initial_balance: i32, epoch_delta: i32, final_balance: i32) -> Self {
        Self {
            initial_balance,
            epoch_delta,
            final_balance,
            _marker: PhantomData,
        }
    }
}

impl<F: Field> ConstraintSynthesizer<F> for EpochBalanceCircuit<F> {
    fn generate_constraints(self, cs: ConstraintSystemRef<F>) -> Result<(), SynthesisError> {
        let sum = self.initial_balance + self.epoch_delta;
        println!("Initial Balance in circuit: {}", self.initial_balance);
        println!("Epoch delta in circuit: {}", self.epoch_delta);
        assert_eq!(sum, self.final_balance);
        Ok(())
    }
}
// Function to generate zk-SNARK proof
pub fn generate_proof(
    initial_balance: i32,
    epoch_delta: i32,
    final_balance: i32,
    proving_key: &ProvingKey<Bn254>,
) -> Proof<Bn254> {
    let circuit = EpochBalanceCircuit::new(initial_balance, epoch_delta, final_balance);

    // Generate the proof using Groth16 with the LibsnarkReduction
    let rng = &mut OsRng;
    Groth16::<Bn254, LibsnarkReduction>::create_random_proof_with_reduction(
        circuit,
        proving_key,
        rng,
    )
    .unwrap()
}
