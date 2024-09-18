use std::time::Instant;

use ark_bn254::Fr;
use ark_r1cs_std::{alloc::AllocVar, fields::fp::FpVar};
use ark_relations::r1cs::ConstraintSystem;
use ark_std::UniformRand;
use ducat::{
    core::{network::Network, org::Organization, transaction::Transaction},
    utils::generate_random_in_range,
};
use rand::Rng;

fn bt_test() {
    let mut rng = ark_std::test_rng();
    let cs = ConstraintSystem::<Fr>::new_ref();

    // Create known addresses for testing

    // Create and add organizations to the network
    let mut network = Network::<Fr>::new();
    let org1 = Organization::new(
        "org1".to_string(),
        20,
        Organization::create_known_addresses(cs.clone(), 5, 0),
    );
    let org2 = Organization::new(
        "org2".to_string(),
        30,
        Organization::create_known_addresses(cs.clone(), 5, 10),
    );
    network.add_organization(org1);
    network.add_organization(org2);

    for i in 1..2 {
        let start = Instant::now();

        // Generate random transaction data
        let tid = FpVar::new_input(cs.clone(), || Ok(Fr::rand(&mut rng))).unwrap();
        let sender_secret = i.to_string();
        let receiver_secret = (i + 10).to_string();
        // let sender_secret = FpVar::new_input(cs.clone(), || Ok(Fr::from(i))).unwrap();
        // let receiver_secret = FpVar::new_input(cs.clone(), || Ok(Fr::rand(&mut rng))).unwrap();
        let sn_secret = FpVar::new_input(cs.clone(), || Ok(generate_random_in_range())).unwrap();

        // Create the original transaction
        let t = Transaction::new(
            tid,
            rand::thread_rng().gen_range(0..100),
            sender_secret.clone(),
            receiver_secret.clone(),
            sn_secret.clone(), // Clone the serial number secret for later use
        );
        println!(
            "Forwarding transaction of value: {} from sender: {:?} to receiver: {:?}",
            t.value(),
            t.sender_address(),
            t.receiver_address()
        );
        // Forward the transaction to the network
        network.forward_transaction(t);

        println!("Nova::prove_step {}: {:?}", i, start.elapsed());
        network.dump_network_info();
        println!("Transferring deltas to org balance");
        network.transfer_delta_to_organization_balance();
        network.dump_network_info();
        network.validate_all_epoch_deltas_and_final_balances();
        network.validate_all_assets();
        network.clean_deltas_at_epoch_end();
    }
}
pub fn main() {
    bt_test();
}
