use ark_bn254::Fr;
use ark_relations::r1cs::ConstraintSystem;
use ducat::core::{org::Organization, run_config::RUN_CONFIG};
use indicatif::ProgressIterator;
#[cfg(feature = "dhat-heap")]
#[global_allocator]
static ALLOC: dhat::Alloc = dhat::Alloc;

pub fn main() {
    #[cfg(feature = "dhat-heap")]
    let _profiler = dhat::Profiler::new_heap();
    std::env::set_var("RUST_BACKTRACE", "full");

    let cs = ConstraintSystem::<Fr>::new_ref();
    // Create organizations
    for i in (0..RUN_CONFIG.org_count).progress() {
        Organization::create_known_addresses(
            &cs,
            RUN_CONFIG.addresses_per_organization,
            i * RUN_CONFIG.addresses_per_organization,
        );
    }
}
