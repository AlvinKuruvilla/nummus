use ark_crypto_primitives::crh::{
    sha256::constraints::{Sha256Gadget, UnitVar},
    CRHSchemeGadget,
};
use ark_ff::PrimeField;
use ark_r1cs_std::{fields::fp::FpVar, ToBytesGadget, ToConstraintFieldGadget};

#[derive(Clone, Debug)]
pub struct Address<F: PrimeField> {
    public_key: FpVar<F>,
    secret_key: FpVar<F>,
}
/// According to the Zcash paper, this is the address generation procedure. Hash the secret key to generate the pairs (pk, sk)
impl<F> Address<F>
where
    F: PrimeField,
{
    pub fn new(secret_key: &FpVar<F>) -> Self {
        // Get the byte representation of the secret key as Vec<UInt8<F>>
        let secret_key_bytes = secret_key.to_bytes().unwrap();

        // Create a UnitVar instance
        let unit_var: UnitVar<F> = UnitVar::default();

        // Use the secret_key_bytes directly in the hash computation
        let sn = &Sha256Gadget::evaluate(&unit_var, &secret_key_bytes)
            .unwrap()
            .0
            .to_constraint_field()
            .unwrap()[0];

        Self {
            public_key: sn.clone(),
            secret_key: secret_key.clone(),
        }
    }
    pub fn public_key(&self) -> &FpVar<F> {
        &self.public_key
    }
    pub fn secret_key(&self) -> &FpVar<F> {
        &self.secret_key
    }
}
