use super::transaction::Transaction;
use crate::{
    gadgets::{
        blockchain_validator::{blockchain_validator_generate_proof, BlockchainValidatorCircuit},
        epoch_circuit::{generate_proof, EpochBalanceCircuit},
    },
    utils::{fpvars_to_u64s, prime_fields_to_u64s},
};
use ark_bn254::{Bn254, Fr};
use ark_crypto_primitives::snark::SNARK;
use ark_ff::PrimeField;
use ark_groth16::{prepare_verifying_key, r1cs_to_qap::LibsnarkReduction, Groth16};
use ark_r1cs_std::fields::fp::FpVar;
use ark_relations::r1cs::ConstraintSystemRef;
use rand::rngs::OsRng;

pub struct Organization<F: PrimeField> {
    // TODO: Change the balance counters to u32 and keep a flag for each of those whether or not they are negative
    spent_serial_numbers: Vec<FpVar<F>>, // TODO: When using this we should have sanity checks that panic if repeats exist
    used_address_public_keys: Vec<String>, // TODO: When using this we should have sanity checks that panic if repeats exist
    transaction_root_cache: Vec<FpVar<F>>,
    unique_identifier: String,
    _initial_balance: i32,
    final_balance: i32,
    epoch_balance_delta: i32,
}
impl<F> Organization<F>
where
    F: PrimeField,
{
    pub fn new(
        unique_identifier: String,
        initial_balance: i32,
        known_addresses: Vec<String>,
    ) -> Self {
        Self {
            spent_serial_numbers: Vec::new(),
            used_address_public_keys: known_addresses,
            unique_identifier,
            transaction_root_cache: Vec::new(),
            // NOTE: this field should not be directly accessed or mutated. There is a getter provided to retrieve the value
            _initial_balance: initial_balance,
            final_balance: initial_balance,
            epoch_balance_delta: 0,
        }
    }
    pub fn add_address_public_key(&mut self, address_public_key: String) {
        self.used_address_public_keys.push(address_public_key);
    }
    pub fn add_serial_number(&mut self, sn: FpVar<F>) {
        self.spent_serial_numbers.push(sn);
    }
    pub fn add_root(&mut self, root: FpVar<F>) {
        self.transaction_root_cache.push(root);
    }
    pub fn clear_delta(&mut self) {
        self.epoch_balance_delta = 0;
    }
    pub fn delta(&self) -> i32 {
        self.epoch_balance_delta
    }
    pub fn final_balance(&self) -> i32 {
        self.final_balance
    }
    pub fn initial_balance(&self) -> i32 {
        self._initial_balance
    }
    pub fn update_delta(&mut self, value: i32) {
        self.epoch_balance_delta += value;
    }
    pub fn update_balance(&mut self, epoch_delta: i32) {
        self.final_balance += epoch_delta;
    }
    pub fn identifier(&self) -> String {
        self.unique_identifier.clone()
    }
    pub fn dump_info(&self) {
        println!("Organization: {}", self.unique_identifier);
        println!("========================");
        println!("Balance: {}", self.final_balance());
        println!("Epoch Delta: {}", self.epoch_balance_delta);
        println!("Known Addresses: {:?}", self.used_address_public_keys);
        println!()
    }
    pub fn has_address(&self, address_public_key: String) -> bool {
        self.used_address_public_keys
            .iter()
            .any(|key| *key == address_public_key)
    }

    pub fn is_involved(&self, t: &Transaction<F>) -> bool {
        self.has_address(t.sender_address().public_key())
            || self.has_address(t.receiver_address().public_key())
    }
    pub fn create_known_addresses(
        cs: ConstraintSystemRef<F>,
        num_addresses: usize,
        offset: usize,
    ) -> Vec<String> {
        let mut addresses = Vec::new();
        for i in 0..num_addresses {
            let address: String = (i + offset).to_string();
            // let address = FpVar::new_input(cs.clone(), || Ok(F::from(i as u64))).unwrap();
            addresses.push(address);
        }
        addresses
    }
    pub fn validate_components(&self, blockchain_keys: Vec<F>, blockchain_values: Vec<F>) {
        let mut rng = OsRng;
        println!(
            "\x1b[32mValidating Organization: {}\x1b[0m",
            self.identifier()
        );

        let (proving_key, verifying_key) =
            Groth16::<Bn254, LibsnarkReduction>::circuit_specific_setup(
                EpochBalanceCircuit::<Fr>::new(
                    self.initial_balance(),
                    self.delta(),
                    self.final_balance(),
                ),
                &mut rng,
            )
            .unwrap();
        let proof = generate_proof(
            self.initial_balance(),
            self.delta(),
            self.final_balance(),
            &proving_key,
        );

        // Prepare the verifying key
        let pvk = prepare_verifying_key(&verifying_key);

        // Verify the proof
        let is_valid =
            Groth16::<Bn254, LibsnarkReduction>::verify_proof(&pvk, &proof, &[]).unwrap();

        println!("Epoch Proof is valid: {}", is_valid);

        let (proving_key, verifying_key) =
            Groth16::<Bn254, LibsnarkReduction>::circuit_specific_setup(
                BlockchainValidatorCircuit::new(
                    prime_fields_to_u64s(blockchain_keys.clone()),
                    prime_fields_to_u64s(blockchain_values.clone()),
                    fpvars_to_u64s(self.transaction_root_cache.clone()),
                    fpvars_to_u64s(self.spent_serial_numbers.clone()),
                ),
                &mut rng,
            )
            .unwrap();
        let proof = blockchain_validator_generate_proof(
            prime_fields_to_u64s(blockchain_keys.clone()),
            prime_fields_to_u64s(blockchain_values.clone()),
            fpvars_to_u64s(self.transaction_root_cache.clone()),
            fpvars_to_u64s(self.spent_serial_numbers.clone()),
            &proving_key,
        );

        // Prepare the verifying key
        let pvk = prepare_verifying_key(&verifying_key);

        // Verify the proof
        let is_valid =
            Groth16::<Bn254, LibsnarkReduction>::verify_proof(&pvk, &proof, &[]).unwrap();
        println!("Blockchain Proof is valid: {}", is_valid);
    }
}
pub fn validate_transaction_serial_numbers(
    blockchain_serial_numbers: Vec<u64>,
    spent_serial_numbers: Vec<u64>,
) -> bool {
    spent_serial_numbers.iter().all(|serial| {
        blockchain_serial_numbers
            .iter()
            .any(|blockchain_serial| *serial == *blockchain_serial)
    })
}
pub fn validate_transaction_roots(
    blockchain_transaction_roots: Vec<u64>,
    transaction_root_cache: Vec<u64>,
) -> bool {
    transaction_root_cache.iter().all(|t_root| {
        blockchain_transaction_roots
            .iter()
            .any(|blockchain_transaction_root| *t_root == *blockchain_transaction_root)
    })
}
