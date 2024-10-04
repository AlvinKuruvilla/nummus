use ark_bn254::Fr;
use ark_ff::{BigInteger, PrimeField};
use ark_r1cs_std::{alloc::AllocVar, fields::fp::FpVar, R1CSVar};
use ark_relations::r1cs::ConstraintSystemRef;
use rand::Rng;

// Helper function to generate a random value in the specified range
pub fn generate_random_in_range() -> Fr {
    let mut rng = rand::thread_rng();
    let ret = Fr::from(rng.gen_range(0..100));
    println!("Generated random serial number secret: {:?}", ret);
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
pub fn prime_field_to_u64<F: PrimeField>(field_element: F) -> Option<u64> {
    // Convert the field element into its underlying representation (little-endian bigint)
    let big_integer = field_element.into_bigint();

    // Extract the lower 64 bits (or fewer, depending on the value)
    let bytes = big_integer.to_bytes_le();

    // Convert the first 8 bytes into a u64 (if available)
    if bytes.len() >= 8 {
        Some(u64::from_le_bytes([
            bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7],
        ]))
    } else {
        // If there are fewer than 8 bytes, you can pad the remaining bytes with zeroes
        let mut padded_bytes = [0u8; 8];
        padded_bytes[..bytes.len()].copy_from_slice(&bytes);
        Some(u64::from_le_bytes(padded_bytes))
    }
}
pub fn fpvar_to_u64<F: PrimeField>(fp_var: &FpVar<F>) -> Option<u64> {
    // Extract the value from FpVar<F> (this is only available if the constraint system has been solved or it's in a trusted setup)
    let result = fp_var.value().unwrap();

    // Convert the PrimeField element to a bigint and then to bytes
    let big_integer = result.into_bigint();
    let bytes = big_integer.to_bytes_le();

    // Extract the first 8 bytes as u64
    if bytes.len() >= 8 {
        Some(u64::from_le_bytes([
            bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7],
        ]))
    } else {
        let mut padded_bytes = [0u8; 8];
        padded_bytes[..bytes.len()].copy_from_slice(&bytes);
        Some(u64::from_le_bytes(padded_bytes))
    }
}
pub fn prime_fields_to_u64s<F: PrimeField>(field_elements: Vec<F>) -> Vec<u64> {
    field_elements
        .into_iter() // Create an iterator over the vector elements
        .map(|fe| prime_field_to_u64(fe).unwrap()) // Apply and unwrap, panic on None
        .collect() // Collect the results into a new vector
}
pub fn fpvars_to_u64s<F: PrimeField>(field_elements: Vec<FpVar<F>>) -> Vec<u64> {
    field_elements
        .into_iter() // Create an iterator over the vector elements
        .map(|fe| fpvar_to_u64(&fe).unwrap()) // Apply and unwrap, panic on None
        .collect()
}
