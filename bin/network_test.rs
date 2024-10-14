use ark_bn254::Fr;
use ark_relations::r1cs::ConstraintSystem;
use ducat::core::{network::Network, org::Organization, run_config::RUN_CONFIG};
use indicatif::ProgressIterator;
use rand::Rng;

pub fn main() {
    std::env::set_var("RUST_BACKTRACE", "full");

    let cs = ConstraintSystem::<Fr>::new_ref();

    // Create and add organizations to the network
    let mut network = Network::<Fr>::new();

    let mut organizations = Vec::new(); // Store organizations for later use

    // Create organizations
    for i in (0..RUN_CONFIG.org_count).progress() {
        let org_name = format!("org{}", i + 1);
        let initial_balance = rand::thread_rng().gen_range(5..500000); // Random initial balance
        let addresses = Organization::create_known_addresses(
            cs.clone(),
            RUN_CONFIG.addresses_per_organization,
            i * RUN_CONFIG.addresses_per_organization,
        );
        let organization = Organization::new(org_name.clone(), initial_balance, addresses.to_vec());
        organizations.push(organization.clone()); // Store organization for later access
        network.add_organization(organization);
    }
}
