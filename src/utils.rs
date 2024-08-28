use ark_bn254::Fr;
use ark_ff::PrimeField;
use ark_r1cs_std::{alloc::AllocVar, fields::fp::FpVar};
use ark_relations::r1cs::ConstraintSystemRef;
use rand::Rng;

// Helper function to generate a random value in the specified range
pub fn generate_random_in_range() -> Fr {
    let mut rng = rand::thread_rng();
    let ret = Fr::from(rng.gen_range(0..100));
    // println!("Generated random serial number secret: {:?}", ret);
    ret
}
/// Convert a string to a field element `F`.
fn string_to_field<F: PrimeField>(s: &str) -> F {
    let mut bytes = s.as_bytes().to_vec();
    // Pad or truncate the byte array to the size of F's modulus.
    let modulus_size = (F::MODULUS_BIT_SIZE / 8) as usize;
    if bytes.len() > modulus_size {
        bytes.truncate(modulus_size);
    } else {
        bytes.resize(modulus_size, 0);
    }

    F::from_le_bytes_mod_order(&bytes)
}

/// Convert a string to an `FpVar<F>`.
pub fn string_to_fpvar<F: PrimeField>(s: String, cs: ConstraintSystemRef<F>) -> FpVar<F> {
    let field_element = string_to_field::<F>(&s);
    FpVar::new_input(cs, || Ok(field_element)).unwrap()
}
