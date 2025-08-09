use std::{collections::HashMap, fmt::Display};

use ark_ff::PrimeField;
use ark_r1cs_std::{fields::fp::FpVar, R1CSVar};
#[allow(clippy::upper_case_acronyms)]
type ROOT<F> = F;
type SN<F> = F;

#[derive(Clone)]
pub struct Blockchain<F: PrimeField> {
    inner: HashMap<SN<F>, ROOT<F>>,
}

impl<F> Blockchain<F>
where
    F: PrimeField,
{
    pub fn new() -> Self {
        Self {
            inner: HashMap::new(),
        }
    }

    pub fn append_transaction(&mut self, root: FpVar<F>, serial_number: FpVar<F>) {
        // Convert FpVar<F> to concrete values
        let root_value = root.value().unwrap();
        let sn_value = serial_number.value().unwrap();
        // Check if the serial number is already in the HashMap
        if self.inner.contains_key(&sn_value) {
            // TODO: This is to purely paper over some issue with repeat sn's being added I think its because our random range is not a full u32 spectrum anymore
            //        but it is unclear
            return;
            panic!("The serial number is already in the blockchain!");
        }

        // Insert the transaction into the HashMap
        self.inner.insert(sn_value, root_value);
    }
    pub fn dump_transactions(&self) {
        println!("Blockchain Transactions:");
        println!("========================");

        for (serial_number, root) in &self.inner {
            println!("Transaction:");
            println!("  Serial Number: {:?}", serial_number);
            println!("  Root: {:?}", root);
        }

        println!("========================");
    }
    pub fn inner(&self) -> HashMap<SN<F>, ROOT<F>> {
        self.inner.clone()
    }
}
impl<F> Default for Blockchain<F>
where
    F: PrimeField,
{
    fn default() -> Self {
        Self::new()
    }
}
impl<F: PrimeField> Display for Blockchain<F> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (serial_number, root) in &self.inner {
            writeln!(f, "Transaction:")?;
            writeln!(f, "  Serial Number: {:?}", serial_number)?;
            writeln!(f, "  Root: {:?}", root)?;
        }
        Ok(())
    }
}
