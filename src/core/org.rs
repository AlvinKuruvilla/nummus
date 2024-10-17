use std::{collections::VecDeque, io::BufRead};

use super::{address::Address, transaction::Transaction};
use crate::{
    core::fiat_transform::generate_alpha,
    gadgets::{
        asset_proof::{count_occurrences, generate_asset_proof, AssetProof},
        blockchain_validator::{blockchain_validator_generate_proof, BlockchainValidatorCircuit},
        epoch_circuit::{generate_proof, EpochBalanceCircuit},
    },
    utils::{fpvars_to_u64s, prime_fields_to_u64s},
};
use ark_bn254::{Bn254, Fr};
use ark_crypto_primitives::snark::SNARK;
use ark_ff::PrimeField;
use ark_groth16::{prepare_verifying_key, r1cs_to_qap::LibsnarkReduction, Groth16};
use ark_r1cs_std::{alloc::AllocVar, eq::EqGadget, fields::fp::FpVar, R1CSVar};
use ark_relations::r1cs::ConstraintSystemRef;
use rand::rngs::OsRng;

#[derive(Clone)]
pub struct Organization<F: PrimeField> {
    // TODO: Change the balance counters to u32 and keep a flag for each of those whether or not they are negative
    spent_serial_numbers: VecDeque<FpVar<F>>,
    used_address_public_keys: Vec<Address<F>>,
    transaction_root_cache: VecDeque<FpVar<F>>,
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
        known_addresses: Vec<Address<F>>,
    ) -> Self {
        Self {
            spent_serial_numbers: VecDeque::new(),
            used_address_public_keys: known_addresses,
            unique_identifier,
            transaction_root_cache: VecDeque::new(),
            // NOTE: this field should not be directly accessed or mutated. There is a getter provided to retrieve the value
            _initial_balance: initial_balance,
            final_balance: initial_balance,
            epoch_balance_delta: 0,
        }
    }
    pub fn add_address_public_key(&mut self, address_public_key: FpVar<F>) {
        if !self.has_address(address_public_key.clone()) {
            self.used_address_public_keys
                .push(Address::new(&address_public_key));
        } else {
            panic!(
                "Repeat address public key {:?} added to used_address_public_keys",
                address_public_key
            );
        }
    }
    pub fn add_serial_number(&mut self, sn: FpVar<F>) {
        if self.has_serial_number(sn.clone()) {
            panic!("Repeat sn added to spent_serial_numbers");
        }
        self.spent_serial_numbers.push_back(sn);
    }
    pub fn known_addresses(&self) -> Vec<Address<F>> {
        self.used_address_public_keys.clone()
    }
    pub fn serial_numbers(&self) -> VecDeque<FpVar<F>> {
        self.spent_serial_numbers.clone()
    }
    pub fn add_root(&mut self, root: FpVar<F>) {
        self.transaction_root_cache.push_back(root);
    }
    pub fn clear_delta(&mut self) {
        self.epoch_balance_delta = 0;
    }
    pub fn clear_final_balance(&mut self) {
        self.final_balance = 0;
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
        println!("Initial Epoch Balance: {}", self.initial_balance());
        println!("Final Epoch Balance: {}", self.final_balance());
        println!("Epoch Delta: {}", self.epoch_balance_delta);
        // println!("Known Addresses: {:?}", self.used_address_public_keys);
        println!();
    }
    pub fn has_address(&self, address_public_key: FpVar<F>) -> bool {
        self.used_address_public_keys.iter().any(|key| {
            key.public_key()
                .is_eq(&address_public_key)
                .unwrap()
                .value()
                .unwrap_or(false)
        })
    }
    pub fn has_serial_number(&self, sn: FpVar<F>) -> bool {
        self.spent_serial_numbers
            .iter()
            .any(|key| key.is_eq(&sn).unwrap().value().unwrap())
    }

    pub fn is_involved(&self, t: &Transaction<F>) -> bool {
        self.has_address(t.sender_address().public_key())
            || self.has_address(t.receiver_address().public_key())
    }
    pub fn create_known_addresses(
        cs: ConstraintSystemRef<F>,
        num_addresses: usize,
        offset: usize,
    ) -> Vec<Address<F>> {
        let mut addresses = Vec::with_capacity(num_addresses);
        let cs = cs.clone();
        for i in 0..num_addresses {
            let input =
                FpVar::new_input(cs.clone(), || Ok(F::from(i as u64 + offset as u64))).unwrap();
            // let size = input.to_bytes().unwrap().len();
            // println!("Address Byte Vector Size: {:?}", size);
            // pause_until_enter();
            let address = Address::new(&input);
            // let address = format!("{}", i + offset); // More descriptive format
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

    pub fn validate_assets(
        &self,
        blockchain_keys: Vec<F>,
        spent_serial_numbers: Vec<u64>,
        blockchain_values: Vec<F>,
    ) {
        let mut rng = OsRng;
        let occurrences: Vec<u32> = count_occurrences(
            prime_fields_to_u64s(blockchain_keys.clone()),
            spent_serial_numbers.clone(),
        )
        .values()
        .copied()
        .collect();
        println!("occurrences: {:?}", occurrences);
        let (proving_key, verifying_key) =
            Groth16::<Bn254, LibsnarkReduction>::circuit_specific_setup(
                AssetProof::new(
                    generate_alpha(blockchain_values),
                    occurrences.clone(),
                    prime_fields_to_u64s(blockchain_keys.clone()),
                    spent_serial_numbers.clone(),
                ),
                &mut rng,
            )
            .unwrap();
        let asset_proof = generate_asset_proof(
            5,
            spent_serial_numbers.clone(),
            prime_fields_to_u64s(blockchain_keys.clone()),
            occurrences,
            &proving_key,
        );
        // Prepare the verifying key
        let pvk = prepare_verifying_key(&verifying_key);

        // Verify the proof
        let is_valid =
            Groth16::<Bn254, LibsnarkReduction>::verify_proof(&pvk, &asset_proof, &[]).unwrap();
        println!("Asset Proof is valid: {}", is_valid);
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
fn pause_until_enter() {
    println!("Press Enter to continue...");
    let stdin = std::io::stdin();
    let _ = stdin.lock().lines().next(); // Wait for user to press Enter
}
