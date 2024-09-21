use ark_ff::PrimeField;
use std::collections::HashMap;

use crate::utils::fpvars_to_u64s;

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
                // I don't think an org needs to do this since they should the address already in their cache
                // Add public keys and serial numbers if they match this organization's records
                // org.add_address_public_key(t.sender_address().public_key());

                // Update balances accordingly
                if org.has_address(t.receiver_address().public_key()) {
                    org.add_serial_number(t.serial_number());
                    org.update_delta(t.value());
                    org.add_root(t.root());
                }
                if org.has_address(t.sender_address().public_key()) {
                    org.add_serial_number(t.serial_number());
                    org.update_delta(-t.value());
                    org.add_root(t.root());
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
        let blockchain_values: Vec<F> = self.blockchain.inner().into_values().collect();

        for org in self.organizations.values_mut() {
            org.validate_components(blockchain_keys.clone(), blockchain_values.clone())
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
}
