use ark_ff::PrimeField;
use ark_r1cs_std::{alloc::AllocVar, eq::EqGadget, fields::fp::FpVar, R1CSVar};
use ark_relations::r1cs::ConstraintSystemRef;
use std::{collections::HashMap, io::BufRead};

use crate::{analysis::estimate_vec_memory_usage_in_gb, utils::fpvars_to_u64s};

use super::{blockchain::Blockchain, org::Organization, transaction::Transaction};

/// The `Network` type is a abstract representation of a cryptocurrency exchange (like FTX or Binance)
#[derive(Clone)]
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
        // TODO: How does this handle a repeat key? For right now it should panic if there is a repeat
        self.organizations.insert(org.identifier().clone(), org);
    }

    pub fn forward_transaction(&mut self, t: Transaction<F>) {
        let binding = t.sender_address();
        let sender_key = binding.public_key();
        let binding = t.receiver_address();
        let receiver_key = binding.public_key();
        let serial_number = t.serial_number();
        let root = t.root();
        let value = t.value();

        // Forward the transaction to all organizations
        for org in self.organizations.values_mut() {
            if org.is_involved(&t) {
                // println!("HERE");
                // pause_until_enter();
                // Check if the organization has either the sender or receiver address
                let has_receiver = org.has_address(receiver_key);
                let has_sender = org.has_address(sender_key);

                if has_receiver {
                    org.add_serial_number(serial_number.clone());
                    org.update_delta(value);
                }
                if has_sender {
                    org.add_serial_number(serial_number.clone());
                    org.update_delta(-value);
                }

                // Add the root only once if either receiver or sender is involved
                if has_receiver || has_sender {
                    org.add_root(root.clone());
                }
            }
        }

        // Add the transaction to the blockchain
        self.blockchain.append_transaction(root, serial_number);
    }

    pub fn dump_network_info(&self) {
        for org in self.organizations.values() {
            org.dump_info();
        }
    }
    pub fn clean_deltas_and_balances_at_epoch_end(&mut self) {
        for org in self.organizations.values_mut() {
            org.clear_delta();
            org.clear_final_balance();
        }
    }
    pub fn transfer_delta_to_organization_balance(&mut self) {
        for org in self.organizations.values_mut() {
            org.update_balance(org.delta());
        }
    }
    pub fn validate_all_epoch_deltas_and_final_balances(&mut self) {
        let blockchain_keys: Vec<F> = self.blockchain.inner().into_keys().collect();
        println!(
            "Blockchain Keys Vector Size for org is {:.6} GB",
            estimate_vec_memory_usage_in_gb(&blockchain_keys)
        );

        let blockchain_values: Vec<F> = self.blockchain.inner().into_values().collect();
        println!(
            "Blockchain Values Vector Size for org is {:.6} GB",
            estimate_vec_memory_usage_in_gb(&blockchain_values)
        );
        // pause_until_enter();
        for org in self.organizations.values_mut() {
            org.validate_components(blockchain_keys.clone(), blockchain_values.clone());
        }
    }
    pub fn validate_all_assets(&mut self) {
        let blockchain_keys: Vec<F> = self.blockchain.inner().into_keys().collect();
        let blockchain_values: Vec<F> = self.blockchain.inner().into_values().collect();
        for org in self.organizations.values_mut() {
            println!("Validating assets for org: {:?}", org.identifier());
            org.validate_assets(
                blockchain_keys.clone(),
                fpvars_to_u64s(org.serial_numbers()),
                blockchain_values.clone(),
            );
        }
    }
    pub fn validate_no_zombie_serial_numbers(&mut self, cs: ConstraintSystemRef<F>) {
        // Convert all blockchain keys to FpVar<F> first
        let blockchain_keys: Vec<FpVar<F>> = self
            .blockchain
            .inner()
            .into_keys()
            .map(|key| FpVar::new_input(cs.clone(), || Ok(key)).unwrap()) // Create FpVar<F> for each key
            .collect();

        // Iterate over each organization and check for zombie serial numbers
        for org in self.organizations.values_mut() {
            let has_zombie_sn = org.unused_serial_numbers().iter().any(|sn| {
                blockchain_keys
                    .iter()
                    .any(|key| sn.is_eq(key).unwrap().value().unwrap())
            });

            if has_zombie_sn {
                panic!("Zombie SN used!");
            }
        }
    }
    pub fn organizations(&self) -> HashMap<String, Organization<F>> {
        self.organizations.clone()
    }
    pub fn blockchain(&self) -> Blockchain<F> {
        self.blockchain.clone()
    }
}
fn pause_until_enter() {
    println!("Press Enter to continue...");
    let stdin = std::io::stdin();
    let _ = stdin.lock().lines().next(); // Wait for user to press Enter
}
