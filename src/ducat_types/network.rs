use ark_bn254::{Bn254, Fr};
use ark_crypto_primitives::snark::SNARK;
use ark_ff::PrimeField;
use ark_groth16::{prepare_verifying_key, r1cs_to_qap::LibsnarkReduction, Groth16};
use rand::rngs::OsRng;
use std::collections::HashMap;

use crate::gadgets::epoch_circuit::{generate_proof, EpochBalanceCircuit};

use super::{blockchain::Blockchain, org::Organization, transaction::Transaction};

pub struct Network<F: PrimeField> {
    organizations: HashMap<String, Organization<F>>, // Maps organization names to their instances
    blockchain: Blockchain<F>,                       // The blockchain where transactions are stored
}
impl<F> Default for Network<F>
where
    F: PrimeField,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<F> Network<F>
where
    F: PrimeField,
{
    pub fn new() -> Self {
        Self {
            organizations: HashMap::new(),
            blockchain: Blockchain::default(),
        }
    }

    pub fn add_organization(&mut self, org: Organization<F>) {
        self.organizations.insert(org.identifier().clone(), org);
    }

    pub fn forward_transaction(&mut self, t: Transaction<F>) {
        // Forward the transaction to all organizations
        for org in self.organizations.values_mut() {
            if org.is_involved(&t) {
                // TODO: I don't think an org needs to do this since they should the address already in their cache
                // Add public keys and serial numbers if they match this organization's records
                // org.add_address_public_key(t.sender_address().public_key());

                // Update balances accordingly
                if org.has_address(t.receiver_address().public_key()) {
                    org.add_serial_number(t.serial_number());
                    org.update_delta(t.value());
                }
                if org.has_address(t.sender_address().public_key()) {
                    org.add_serial_number(t.serial_number());
                    org.update_delta(-t.value());
                }
            }
        }

        // Add the transaction to the blockchain
        self.blockchain
            .append_transaction(t.root(), t.serial_number());
    }

    pub fn dump_network_info(&self) {
        for org in self.organizations.values() {
            org.dump_info();
        }
    }
    pub fn clean_deltas_at_epoch_end(&mut self) {
        for org in self.organizations.values_mut() {
            org.clear_delta();
        }
    }
    pub fn transfer_delta_to_organization_balance(&mut self) {
        for org in self.organizations.values_mut() {
            org.update_balance(org.delta());
        }
    }
    pub fn validate_all_epoch_deltas_and_final_balances(&mut self) {
        let mut rng = OsRng;
        for org in self.organizations.values_mut() {
            println!(
                "\x1b[32mValidating Organization: {}\x1b[0m",
                org.identifier()
            );
            let (proving_key, verifying_key) =
                Groth16::<Bn254, LibsnarkReduction>::circuit_specific_setup(
                    EpochBalanceCircuit::<Fr>::new(
                        org.initial_balance(),
                        org.delta(),
                        org.final_balance(),
                    ),
                    &mut rng,
                )
                .unwrap();
            let proof = generate_proof(
                org.initial_balance(),
                org.delta(),
                org.final_balance(),
                &proving_key,
            );

            // Prepare the verifying key
            let pvk = prepare_verifying_key(&verifying_key);

            // Verify the proof
            let is_valid =
                Groth16::<Bn254, LibsnarkReduction>::verify_proof(&pvk, &proof, &[]).unwrap();

            println!("Proof is valid: {}", is_valid);
        }
    }
}
