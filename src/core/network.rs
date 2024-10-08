use ark_ff::PrimeField;
use std::collections::HashMap;

use crate::utils::fpvars_to_u64s;

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
        let sender_key = t.sender_address().public_key();
        let receiver_key = t.receiver_address().public_key();
        let serial_number = t.serial_number();
        let root = t.root();
        let value = t.value();

        // Forward the transaction to all organizations
        for org in self.organizations.values_mut() {
            if org.is_involved(&t) {
                // Check if the organization has either the sender or receiver address
                let has_receiver = org.has_address(receiver_key.clone());
                let has_sender = org.has_address(sender_key.clone());

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
        let blockchain_values: Vec<F> = self.blockchain.inner().into_values().collect();

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
}
