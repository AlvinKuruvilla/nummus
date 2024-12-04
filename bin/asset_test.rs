use ark_bn254::Fr;
use ark_r1cs_std::{alloc::AllocVar, fields::fp::FpVar};
use ark_relations::r1cs::ConstraintSystem;
use ark_std::{test_rng, UniformRand};
use ducat::{
    core::{network::Network, org::Organization, run_config::RUN_CONFIG, transaction::Transaction},
    utils::generate_random_in_range,
};
use rand::{seq::IteratorRandom, Rng};
use std::time::Instant;

pub fn main() {
    std::env::set_var("RUST_BACKTRACE", "full");

    let mut rng = test_rng();
    let cs = ConstraintSystem::<Fr>::new_ref();

    // Create and add organizations to the network
    let mut network = Network::<Fr>::new();

    let mut organizations = Vec::new(); // Store organizations for later use

    // Create organizations
    // TODO: At the beginning of each new epoch should we change something about the organization's initial balances or their addresses?
    //       This may not necessarily be needed of we consider each run their oen epoch
    for i in 0..RUN_CONFIG.org_count {
        let org_name = format!("org{}", i + 1);
        let initial_balance = rand::thread_rng().gen_range(5..500000); // Random initial balance
        let addresses = Organization::create_known_addresses(
            &cs,
            RUN_CONFIG.addresses_per_organization,
            i * RUN_CONFIG.addresses_per_organization,
        );
        let organization = Organization::new(org_name.clone(), initial_balance, addresses.to_vec());
        organizations.push(organization.clone()); // Store organization for later access
        network.add_organization(organization);
    }

    // Generate random transaction data
    for _ in 0..RUN_CONFIG.transaction_count {
        let tid = FpVar::new_input(cs.clone(), || Ok(Fr::rand(&mut rng))).unwrap();

        // Select random sender and receiver organizations
        let sender_index = rand::thread_rng().gen_range(0..RUN_CONFIG.org_count);
        let receiver_index = rand::thread_rng().gen_range(0..RUN_CONFIG.org_count);

        // Ensure sender and receiver are different
        let receiver_index = if sender_index == receiver_index {
            (receiver_index + 1) % RUN_CONFIG.org_count
        } else {
            receiver_index
        };

        // Select random addresses from the organizations
        let sender_address = organizations[sender_index]
            .known_addresses() // Returns a `&HashSet<String>`
            .iter() // Create an iterator over the addresses
            .choose(&mut rng)
            .cloned()
            .unwrap();
        let receiver_address = organizations[receiver_index]
            .known_addresses() // Returns a `&HashSet<String>`
            .iter() // Create an iterator over the addresses
            .choose(&mut rng)
            .cloned()
            .unwrap();
        let sn_secret = FpVar::new_input(cs.clone(), || Ok(generate_random_in_range())).unwrap();

        // Create the transaction
        let value = rand::thread_rng().gen_range(0..100);
        let transaction = Transaction::new(tid, value, sender_address, receiver_address, sn_secret);

        // println!(
        //     "Forwarding transaction of value: {} from sender: {:?} to receiver: {:?}",
        //     transaction.value(),
        //     transaction.sender_address(),
        //     transaction.receiver_address()
        // );

        // Forward the transaction to the network
        network.forward_transaction(transaction);
    }
    let start = Instant::now();
    // network.dump_network_info();
    println!("Transferring deltas to org balance");
    network.transfer_delta_to_organization_balance();
    // network.dump_network_info();
    // network.validate_all_epoch_deltas_and_final_balances();
    //* Uncomment the below line and comment out the above line when we are testing just the asset proof */
    //* If the validate_all_epoch_deltas_and_final_balances is not commented out but validate_all_assets is commented out we are just testing the epoch proof  */
    network.validate_all_assets();
    network.clean_deltas_and_balances_at_epoch_end();
    println!("Nova::prove_step: {:?}", start.elapsed());
}
