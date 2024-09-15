use ark_crypto_primitives::crh::{
    sha256::constraints::{Sha256Gadget, UnitVar},
    CRHSchemeGadget,
};
use ark_ff::PrimeField;
use ark_r1cs_std::{fields::fp::FpVar, ToBytesGadget, ToConstraintFieldGadget};

/// A serial number for a transaction.
/// According to the zcash paper, the user u first samples ρ, which is a secret value that determines the coin’s serial number as
/// sn = hash(p)
/// For our purposes, 'value' encapsulates the result of 'hash(p)'
/// NOTE: The zcash specification defines their own methodologies for generating spending keys  which are the spiritual successor to the
/// double-spending protection serial numbers provide
pub struct TransactionSerialNumber<F: PrimeField> {
    value: FpVar<F>,
}
impl<F> TransactionSerialNumber<F>
where
    F: PrimeField,
{
    pub fn new(p: FpVar<F>) -> Self {
        let mut holder = vec![];
        let unit_var: UnitVar<F> = UnitVar::default();
        holder.extend_from_slice(&p.to_bytes().unwrap());

        let sn = Sha256Gadget::evaluate(&unit_var, &holder)
            .unwrap()
            .0
            .to_constraint_field()
            .unwrap()[0]
            .to_owned();
        Self { value: sn }
    }
    pub fn sn(&self) -> FpVar<F> {
        self.value.clone()
    }
}
