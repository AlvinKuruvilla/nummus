import sys
import os
import numpy as np
import logging

sys.path.append(os.path.dirname(os.path.abspath(__file__)))

from experiments import (
    ADDRESSES_PER_ORGANIZATION_DEFAULT,
    ORG_COUNT_DEFAULT,
    TRANSACTION_COUNT_DEFAULT,
)
from process_helper import (
    ConfigKey,
    ProofType,
    build_executables,
    enum_to_alias,
    modify_key,
    extract_time,
    reset_to_default_config_state,
    run_benchmark_and_get_proof_time,
)

logging.basicConfig(filename="key_max_test.log", level=logging.INFO)


def config_key_to_alias(key: ConfigKey):
    if key == ConfigKey.ORG_COUNT:
        return "org_key"
    elif key == ConfigKey.ADDRESSES_PER_ORGANIZATION:
        return "address_key"
    elif key == ConfigKey.TRANSACTION_COUNT:
        return "transaction_key"


def find_max_key_distribution(
    proof_type: ProofType, config_key: ConfigKey, initial_increment=10, fine_increment=1
):
    org_key = 50
    crashed = False
    build_executables()
    # Initial loop with larger increments to find crash point
    while not crashed:
        try:
            logging.info(f"Testing {config_key_to_alias(config_key)}: {org_key}")
            print(f"Testing {config_key_to_alias(config_key)}: {org_key}")
            modify_key(config_key, org_key)
            time = extract_time(
                run_benchmark_and_get_proof_time(enum_to_alias(proof_type))
            )
            logging.info(
                f"Success with {config_key_to_alias(config_key)}: {org_key} with time {time}"
            )
            print(
                f"Success with {config_key_to_alias(config_key)}: {org_key} with time {time}"
            )

            # Increment org_key for the next test
            org_key += initial_increment
        except ValueError:
            crashed = True
            logging.error(
                f"Process killed at {config_key_to_alias(config_key)}: {org_key}"
            )
            print(f"Process killed at {config_key_to_alias(config_key)}: {org_key}")
            # Reset to last working configuration for fine-tuning
            org_key -= initial_increment
            break

    # Fine-tuning loop with smaller increments to get closer to max
    crashed = False
    while not crashed:
        try:
            logging.info(
                f"Fine-tuning with {config_key_to_alias(config_key)}: {org_key}"
            )
            print(f"Fine-tuning with {config_key_to_alias(config_key)}: {org_key}")
            modify_key(config_key, org_key)
            fine_tuned_time = extract_time(
                run_benchmark_and_get_proof_time(enum_to_alias(proof_type))
            )
            logging.info(
                f"Success with {config_key_to_alias(config_key)}: {org_key} with time: {fine_tuned_time}"
            )
            print(
                f"Success with {config_key_to_alias(config_key)}: {org_key} with time: {fine_tuned_time}"
            )

            # Increment org_key with fine increment
            org_key += fine_increment
        except ValueError:
            crashed = True
            logging.error(
                f"Process killed during fine-tuning at {config_key_to_alias(config_key)}: {org_key}"
            )
            print(
                f"Process killed during fine-tuning at {config_key_to_alias(config_key)}: {org_key}"
            )
            # Backtrack by one fine increment to get the maximum safe value
            org_key -= fine_increment
            logging.info(
                f"Maximum stable {config_key_to_alias(config_key)} found: {org_key}"
            )
            print(f"Maximum stable {config_key_to_alias(config_key)}: {org_key}")
            break

    # Use logarithmic spacing to generate a distribution up to the max org_key
    org_keys_distribution = (
        np.round(np.logspace(np.log10(10), np.log10(org_key), 5)).astype(int).tolist()
    )
    logging.info(
        f"Generated keys distribution: {org_keys_distribution} for {enum_to_alias(proof_type)} and config key {config_key_to_alias(config_key)}"
    )
    print(
        f"Optimal keys distribution: {org_keys_distribution} for {enum_to_alias(proof_type)} and config key {config_key_to_alias(config_key)}"
    )
    # Write results to file
    with open("macbook_sequences.txt", "a") as f:
        f.write(
            f"Proof Type: {enum_to_alias(proof_type)}, ConfigKey: {config_key_to_alias(config_key)}\n"
        )
        f.write(f"org_keys distribution: {org_keys_distribution}\n\n")
    reset_to_default_config_state(
        ORG_COUNT_DEFAULT, TRANSACTION_COUNT_DEFAULT, ADDRESSES_PER_ORGANIZATION_DEFAULT
    )
    return org_keys_distribution


print("Starting org_key maxing for all proof types")
find_max_key_distribution(ProofType.ALL, ConfigKey.ORG_COUNT)
find_max_key_distribution(ProofType.ASSET, ConfigKey.ORG_COUNT)
find_max_key_distribution(ProofType.EPOCH, ConfigKey.ORG_COUNT)

print("Starting address_key maxing for all proof types")
find_max_key_distribution(ProofType.ALL, ConfigKey.ADDRESSES_PER_ORGANIZATION)
find_max_key_distribution(ProofType.ASSET, ConfigKey.ADDRESSES_PER_ORGANIZATION)
find_max_key_distribution(ProofType.EPOCH, ConfigKey.ADDRESSES_PER_ORGANIZATION)

print("Starting transaction_key maxing for all proof types")
find_max_key_distribution(ProofType.ALL, ConfigKey.TRANSACTION_COUNT)
find_max_key_distribution(ProofType.ASSET, ConfigKey.TRANSACTION_COUNT)
find_max_key_distribution(ProofType.EPOCH, ConfigKey.TRANSACTION_COUNT)
