use ark_ff::PrimeField;
use ark_r1cs_std::{alloc::AllocVar, fields::fp::FpVar};
use ark_relations::r1cs::ConstraintSystem;
use rand::thread_rng;

use crate::{gadgets::merkle_gadget::MerkleTreeGadget, utils::string_to_fpvar};

use super::{address::Address, serial_number::TransactionSerialNumber};

pub struct Transaction<F: PrimeField> {
    transaction_id: FpVar<F>,
    value: i32,
    sender_address: Address,   // built from the spending key of the sender
    receiver_address: Address, // built from the spending key of the receiver
    serial_number: TransactionSerialNumber<F>,
}
impl<F> Transaction<F>
where
    F: PrimeField,
{
    pub fn new(
        transaction_id: FpVar<F>,
        value: i32,
        sender_address_secret: String,
        receiver_address_secret: String,
        sn_secret: FpVar<F>,
    ) -> Self {
        Self {
            transaction_id,
            value,
            sender_address: Address::new(sender_address_secret),
            receiver_address: Address::new(receiver_address_secret),
            serial_number: TransactionSerialNumber::new(sn_secret),
        }
    }
    pub fn transaction_id(&self) -> FpVar<F> {
        self.transaction_id.clone()
    }
    pub fn value(&self) -> i32 {
        self.value
    }
    pub fn value_as_field_element(&self) -> FpVar<F> {
        let cs = ConstraintSystem::<F>::new_ref();
        let field_element = F::from(self.value() as u64);
        FpVar::<F>::new_input(cs, || Ok(field_element)).unwrap()
    }
    pub fn sender_address(&self) -> Address {
        self.sender_address.clone()
    }
    pub fn receiver_address(&self) -> Address {
        self.receiver_address.clone()
    }
    pub fn serial_number(&self) -> FpVar<F> {
        self.serial_number.sn()
    }
    pub fn to_vec(&self) -> Vec<FpVar<F>> {
        let cs = ConstraintSystem::<F>::new_ref();
        vec![
            self.transaction_id(),
            self.value_as_field_element(),
            string_to_fpvar(self.sender_address().public_key(), cs.clone()),
            string_to_fpvar(self.sender_address().secret_key(), cs.clone()),
            string_to_fpvar(self.receiver_address().public_key(), cs.clone()),
            string_to_fpvar(self.receiver_address().secret_key(), cs.clone()),
            self.serial_number(),
        ]
    }
    pub fn root(&self) -> FpVar<F> {
        let leaves = self.to_vec();
        let cs = ConstraintSystem::<F>::new_ref();

        for (idx, _) in leaves.iter().enumerate() {
            // Validate each leaf
            let is_valid =
                MerkleTreeGadget::generate_proof_and_validate(&leaves, cs.clone(), vec![idx]);
            if !is_valid {
                panic!("Cannot get root hash if leaves are not all valid");
            }
        }
        MerkleTreeGadget::create_root_hash(leaves, cs)
    }
    /// This assumes a single split where the remainder is given back to the original person
    pub fn split_transaction(
        &self,
        split_values: Vec<i32>,               // The values to split into
        new_receiver_addresses: Vec<Address>, // The new receiver addresses for each split
        sender_address_secret: String,        // Sender's secret key
    ) -> Vec<Self> {
        // Ensure that the split values sum up to the original transaction value
        let cs = ConstraintSystem::<F>::new_ref();
        let mut total_split_value = 0;

        for split_value in &split_values {
            total_split_value += split_value;
        }

        // Enforce that the sum of the split values equals the original transaction value
        assert_eq!(total_split_value, self.value());

        // Create the split transactions
        let mut split_transactions = Vec::new();
        let mut rng = thread_rng();

        for (i, split_value) in split_values.into_iter().enumerate() {
            let new_transaction_id =
                FpVar::<F>::new_input(cs.clone(), || Ok(F::rand(&mut rng))).unwrap();

            // Create a new serial number for the split transaction
            let new_serial_number = TransactionSerialNumber::new(
                FpVar::new_input(cs.clone(), || Ok(F::rand(&mut rng))).unwrap(),
            );

            // Generate the new split transaction
            let split_transaction = Transaction {
                transaction_id: new_transaction_id,
                value: split_value,
                sender_address: Address::new(sender_address_secret.clone()), // Sender remains the same
                receiver_address: new_receiver_addresses[i].clone(), // New receiver address for this split
                serial_number: new_serial_number,
            };

            split_transactions.push(split_transaction);
        }

        split_transactions
    }
}
