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
pub fn main() {
    std::env::set_var("RUST_BACKTRACE", "full");

    let mut rng = ark_std::test_rng();
    let cs = ConstraintSystem::<Fr>::new_ref();
    let i = 1;
    // Create and add organizations to the network
    let mut network = Network::<Fr>::new();

    let start = Instant::now();
    // Organization setup
    // TODO: At the beginning of each new epoch should we change something about the organization's initial balances or their addresses?
    let org1 = Organization::new(
        "org1".to_string(),
        20,
        Organization::create_known_addresses(cs.clone(), 20, 0),
    );
    let org2 = Organization::new(
        "org2".to_string(),
        30,
        Organization::create_known_addresses(cs.clone(), 20, 10),
    );
    network.add_organization(org1);
    network.add_organization(org2);

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
        sn_secret.clone(),
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
    network.clean_deltas_and_balances_at_epoch_end();
}
