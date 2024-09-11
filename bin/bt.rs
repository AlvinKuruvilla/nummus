use ark_bn254::Fr;
use ark_crypto_primitives::crh::{
    sha256::constraints::{Sha256Gadget, UnitVar},
    CRHSchemeGadget,
};
use ark_r1cs_std::ToBytesGadget;
use ark_r1cs_std::ToConstraintFieldGadget;
use ark_r1cs_std::{alloc::AllocVar, fields::fp::FpVar};
use ark_relations::r1cs::ConstraintSystem;
use ark_std::UniformRand;
use ducat::{
    ducat_types::{
        address::Address, blockchain::Blockchain, org::Organization, transaction::Transaction,
    },
    utils::generate_random_in_range,
};
use rand::Rng;

use std::time::Instant;
fn bt_test() {
    let mut blockchain = Blockchain::default();
    let cs = ConstraintSystem::<Fr>::new_ref();
    let mut rng = ark_std::test_rng();
    let mut org1 = Organization::new(
        "org1".to_string(),
        20,
        Organization::create_known_addresses(cs.clone(), 5, 0),
    );
    let mut org2 = Organization::new(
        "org2".to_string(),
        30,
        Organization::create_known_addresses(cs.clone(), 5, 10),
    );
    for i in 1..5 {
        let start = Instant::now();
        let cs = ConstraintSystem::<Fr>::new_ref();

        // Generate random transaction data
        let tid = FpVar::new_input(cs.clone(), || Ok(Fr::rand(&mut rng))).unwrap();
        let sender_secret = FpVar::new_input(cs.clone(), || Ok(Fr::rand(&mut rng))).unwrap();
        let receiver_secret = FpVar::new_input(cs.clone(), || Ok(Fr::rand(&mut rng))).unwrap();
        let sn_secret = FpVar::new_input(cs.clone(), || Ok(generate_random_in_range())).unwrap();

        // Create the original transaction
        let t = Transaction::new(
            tid,
            rand::thread_rng().gen_range(0..100),
            sender_secret.clone(),
            receiver_secret.clone(),
            sn_secret.clone(), // Clone the serial number secret for later use
        );
        // TODO: Appending to the address_public_key cache isn't great. Rather we should have some sort of forwarding method that sends all transactions to all
        // organizations for them to check the sender and receiver address against their existing set of addresses.
        org1.add_address_public_key(t.sender_address().public_key());
        org1.add_serial_number(t.serial_number());
        org2.add_address_public_key(t.receiver_address().public_key());
        blockchain.append_transaction(t.root(), t.serial_number());
        println!("Nova::prove_step {}: {:?}", i, start.elapsed());
    }
    blockchain.dump_transactions();
}
fn bt_test_with_split_transactions() {
    let mut blockchain = Blockchain::default();
    let mut rng = ark_std::test_rng();

    for i in 1..5 {
        let start = Instant::now();
        let cs = ConstraintSystem::<Fr>::new_ref();

        // Generate random transaction data
        let tid = FpVar::new_input(cs.clone(), || Ok(Fr::rand(&mut rng))).unwrap();
        let sender_secret = FpVar::new_input(cs.clone(), || Ok(Fr::rand(&mut rng))).unwrap();
        let receiver_secret_1 = FpVar::new_input(cs.clone(), || Ok(Fr::rand(&mut rng))).unwrap();
        let receiver_secret_2 = FpVar::new_input(cs.clone(), || Ok(Fr::rand(&mut rng))).unwrap();
        let sn_secret = FpVar::new_input(cs.clone(), || Ok(generate_random_in_range())).unwrap();

        // Create the original transaction
        let t = Transaction::new(
            tid,
            rand::thread_rng().gen_range(0..100),
            sender_secret.clone(),
            receiver_secret_1.clone(),
            sn_secret.clone(), // Clone the serial number secret for later use
        );

        // Define the split values and new receiver addresses
        let split_values = vec![
            rand::thread_rng().gen_range(0..100),
            rand::thread_rng().gen_range(0..100),
        ];
        let new_receiver_addresses = vec![
            Address::new(receiver_secret_1.clone()),
            Address::new(receiver_secret_2.clone()),
        ];

        // Perform the transaction split

        // NOTE: Since new serial numbers are generated on split transactions using thread_rng(), we are much less likely to get an overlap
        // However, as the function above shows, if you do not consider transaction splits, and bound the rng to a very small range, we will get the expected panics when appending transactions
        let split_transactions =
            t.split_transaction(split_values, new_receiver_addresses, sender_secret);

        // Append each split transaction to the blockchain
        for split_tx in split_transactions {
            blockchain.append_transaction(split_tx.root(), split_tx.serial_number());
        }

        println!("Nova::prove_step {}: {:?}", i, start.elapsed());
    }
    blockchain.dump_transactions();
    // After the loop, introduce a duplicate serial number deliberately
    let cs = ConstraintSystem::<Fr>::new_ref();
    let tid = FpVar::new_input(cs.clone(), || Ok(Fr::rand(&mut rng))).unwrap();
    let value = FpVar::new_input(cs.clone(), || Ok(generate_random_in_range())).unwrap();
    let sender_secret = FpVar::new_input(cs.clone(), || Ok(Fr::rand(&mut rng))).unwrap();
    let receiver_secret = FpVar::new_input(cs.clone(), || Ok(Fr::rand(&mut rng))).unwrap();
    let sn_secret = FpVar::new_input(cs.clone(), || Ok(Fr::from(110))).unwrap();

    // let extra_tx = Transaction::new(
    //     tid.clone(),
    //     value.clone(),
    //     sender_secret.clone(),
    //     receiver_secret.clone(),
    //     sn_secret.clone(),
    // );

    // blockchain.append_transaction(extra_tx.root(), extra_tx.serial_number());

    // let dupe_tx = Transaction::new(
    //     tid,
    //     value,
    //     sender_secret.clone(),
    //     receiver_secret.clone(),
    //     sn_secret.clone(), // Deliberately reuse the same serial number secret to trigger the expected panic
    // );
    // blockchain.append_transaction(dupe_tx.root(), dupe_tx.serial_number());
    // println!(
    //     "This line should not be reached if the duplicate serial number check works correctly."
    // );
}
pub fn serial_number_test() {
    let cs = ConstraintSystem::<Fr>::new_ref();
    let value = generate_random_in_range(); // Let's assume this returns a fixed value like BigInt([1, 0, 0, 0])
    let fp_var = FpVar::new_input(cs.clone(), || Ok(value)).unwrap();

    let mut holder = vec![];
    holder.extend_from_slice(&fp_var.to_bytes().unwrap());

    // println!("Byte representation: {:?}", holder);

    let unit_var: UnitVar<Fr> = UnitVar::default();
    let sn = Sha256Gadget::evaluate(&unit_var, &holder)
        .unwrap()
        .0
        .to_constraint_field()
        .unwrap()[0]
        .to_owned();
    println!("Generated serial number: {:?}", sn);
    println!("Generated random serial number secret: {:?}", value);
}
pub fn main() {
    bt_test();
}
