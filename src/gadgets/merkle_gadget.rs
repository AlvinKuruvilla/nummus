use ark_ff::{BigInteger, PrimeField};
use ark_r1cs_std::{alloc::AllocVar, fields::fp::FpVar, R1CSVar, ToBytesGadget};
use ark_relations::r1cs::ConstraintSystemRef;
use rs_merkle::MerkleProof;
use rs_merkle::{algorithms::Sha256, Hasher, MerkleTree};
pub struct MerkleTreeGadget;

impl MerkleTreeGadget {
    pub fn create_root_hash_from_scalar_fields<F: PrimeField>(leaves: Vec<F>) -> F {
        // Use rs-merkle to create a Merkle tree and get the root hash
        let leaf_hashes: Vec<[u8; 32]> = leaves
            .iter()
            .map(|leaf| {
                let bytes = leaf.into_bigint().to_bytes_le();
                let mut hash_input = [0u8; 32];
                hash_input[..bytes.len()].copy_from_slice(&bytes);
                Sha256::hash(&hash_input)
            })
            .collect();

        let tree = MerkleTree::<Sha256>::from_leaves(&leaf_hashes);
        // Convert the root hash to the PrimeField element
        F::from_be_bytes_mod_order(&tree.root().unwrap())
    }
    pub fn create_root_hash<F: PrimeField>(
        leaves: Vec<FpVar<F>>,
        cs: ConstraintSystemRef<F>,
    ) -> FpVar<F> {
        // Pre-allocate memory for the leaf hashes vector
        let mut leaf_hashes: Vec<[u8; 32]> = Vec::with_capacity(leaves.len());

        for leaf in leaves.iter() {
            let bytes = leaf.to_bytes().unwrap();
            let mut hash_input = [0u8; 32];

            // Directly copy bytes into the hash_input array, avoiding intermediate Vec<u8>
            for (i, byte) in bytes.iter().enumerate() {
                hash_input[i] = byte.value().unwrap();
            }

            // Compute the hash and push it to the pre-allocated vector
            let hash = Sha256::hash(&hash_input);
            let mut hash_array = [0u8; 32];
            hash_array.copy_from_slice(&hash);
            leaf_hashes.push(hash_array);
        }

        // Create the Merkle tree from leaf hashes
        let tree = MerkleTree::<Sha256>::from_leaves(&leaf_hashes);

        // Get the root hash of the Merkle tree
        let root_hash = tree.root().unwrap();

        // Convert the root hash to a field element
        let root_hash_field = F::from_le_bytes_mod_order(&root_hash);

        // Convert the root hash field element to an FpVar
        FpVar::new_witness(cs, || Ok(root_hash_field)).unwrap()
    }
    pub fn create_merkle_tree<F: PrimeField>(
        leaves: Vec<FpVar<F>>,
        cs: ConstraintSystemRef<F>,
    ) -> MerkleTree<Sha256> {
        // Pre-allocate memory for the leaf hashes
        let mut leaf_hashes: Vec<[u8; 32]> = Vec::with_capacity(leaves.len());

        for leaf in leaves.iter() {
            let bytes = leaf.to_bytes().unwrap();
            let mut hash_input = [0u8; 32];

            // Directly copy the byte values to the hash input array
            for (i, byte) in bytes.iter().enumerate() {
                hash_input[i] = byte.value().unwrap();
            }

            // Compute the hash and push it to the pre-allocated vector
            let hash = Sha256::hash(&hash_input);
            let mut hash_array = [0u8; 32];
            hash_array.copy_from_slice(&hash);
            leaf_hashes.push(hash_array);
        }

        // Create the Merkle tree from leaf hashes
        MerkleTree::<Sha256>::from_leaves(&leaf_hashes)
    }

    pub fn generate_proof_and_validate<F: PrimeField>(
        leaves: &[FpVar<F>],
        indices_to_prove: Vec<usize>,
    ) -> bool {
        // Pre-allocate memory for the leaf hashes
        let mut leaf_hashes: Vec<[u8; 32]> = Vec::with_capacity(leaves.len());

        // Compute leaf hashes once and store them for both tree creation and proof validation
        for leaf in leaves.iter() {
            let bytes = leaf.to_bytes().unwrap();

            // Collect the bytes directly into the hash and compute the hash
            let hash = Sha256::hash(
                &bytes
                    .iter()
                    .map(|byte| byte.value().unwrap())
                    .collect::<Vec<u8>>(),
            );

            // Push the hash into the pre-allocated vector
            leaf_hashes.push(hash);
        }
        // Create the Merkle tree from the pre-computed leaf hashes
        let tree = MerkleTree::<Sha256>::from_leaves(&leaf_hashes);

        // Collect the leaves that need to be proven
        let leaves_to_prove: Vec<_> = indices_to_prove
            .iter()
            .filter_map(|&i| leaf_hashes.get(i))
            .cloned() // Clone the values if necessary
            .collect();

        // Generate the Merkle proof
        let merkle_proof = tree.proof(&indices_to_prove);
        let merkle_root = tree.root().unwrap();

        // Serialize proof to pass it to the client
        let proof_bytes = merkle_proof.to_bytes();

        // Parse proof back on the client
        let proof = MerkleProof::<Sha256>::try_from(proof_bytes).unwrap();

        // Verify the proof
        let ret = proof.verify(
            merkle_root,
            &indices_to_prove,
            &leaves_to_prove,
            leaves.len(),
        );
        assert!(ret);
        ret
    }
}
