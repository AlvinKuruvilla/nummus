use ark_ff::PrimeField;
use ark_r1cs_std::fields::fp::FpVar;

use super::serial_number::TransactionSerialNumber;

pub struct Organization<F: PrimeField> {
    spent_serial_numbers: Vec<TransactionSerialNumber<F>>, // TODO: When using this we should have sanity checks that panic if repeats exist
    used_address_public_keys: Vec<FpVar<F>>, // TODO: When using this we should have sanity checks that panic if repeats exist
    unique_identifier: String,
}
impl<F> Organization<F>
where
    F: PrimeField,
{
    pub fn new(unique_identifier: String) -> Self {
        Self {
            spent_serial_numbers: Vec::new(),
            used_address_public_keys: Vec::new(),
            unique_identifier,
        }
    }
    pub fn add_address_public_key(&mut self, address_public_key: FpVar<F>) {
        self.used_address_public_keys.push(address_public_key);
    }
    pub fn add_serial_number(&mut self, sn: TransactionSerialNumber<F>) {
        self.spent_serial_numbers.push(sn);
    }
}
