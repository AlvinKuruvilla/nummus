import subprocess
import enum
import json
from tabulate import tabulate


class ConfigKey(enum.Enum):
    ORG_COUNT = 1
    TRANSACTION_COUNT = 2
    ADDRESSES_PER_ORGANIZATION = 3


class ProofType(enum.Enum):
    EPOCH = 1
    ASSET = 2
    ALL = 3


def run_benchmark_and_get_proof_time(binary_name: str) -> str:
    _ = subprocess.run(
        ["cargo", "build", "--release", "--examples"],
        stdout=subprocess.DEVNULL,
        stderr=subprocess.STDOUT,
    )
    path = "target/release/examples/" + binary_name
    result = subprocess.run([path], stdout=subprocess.PIPE)
    out = result.stdout.decode("utf-8").split("\n")
    return out[len(out) - 2]


def extract_time(nova_string: str) -> float:
    # Use a regular expression to find the time value
    prefix = "Nova::prove_step: "
    try:
        time = float(nova_string[len(prefix) : -1])
        return time
    except ValueError:
        # This hopefully only triggers when the proof time is in milliseconds
        # so we can just remove an extra character.
        # We have to divide by 1000 here because we know the time is in ms
        return float(nova_string[len(prefix) : -2]) / 1000


def modify_key(key: ConfigKey, new_value: int):
    with open("run_config.json", "r") as file:
        config = json.load(file)

    # Modify the corresponding key based on the Enum value
    if key == ConfigKey.ORG_COUNT:
        config["org_count"] = new_value
    elif key == ConfigKey.TRANSACTION_COUNT:
        config["transaction_count"] = new_value
    elif key == ConfigKey.ADDRESSES_PER_ORGANIZATION:
        config["addresses_per_organization"] = new_value
    else:
        raise ValueError("Invalid key provided.")

    # Write the updated configuration back to the JSON file
    with open("run_config.json", "w") as file:
        json.dump(config, file, indent=4)


def reset_to_default_config_state(org_count, transaction_count, address_count):
    modify_key(ConfigKey.ORG_COUNT, org_count)
    modify_key(ConfigKey.TRANSACTION_COUNT, transaction_count)
    modify_key(ConfigKey.ADDRESSES_PER_ORGANIZATION, address_count)


def enum_to_alias(enum_member):
    if enum_member == ProofType.EPOCH:
        return "et"
    elif enum_member == ProofType.ASSET:
        return "at"
    elif enum_member == ProofType.ALL:
        return "pt"
    else:
        raise ValueError("Unknown Enum member")


def print_results_table(keys, times, proof_type: ProofType):
    if len(keys) != len(times):
        raise ValueError("Both lists must have the same length")

    # Prepare the table headers
    headers = ["Key", enum_to_alias(proof_type) + "_" + "Exec Time"]

    # Combine the two lists into a list of tuples (key, value)
    rows = list(zip(keys, times))

    # Print the table
    print(tabulate(rows, headers=headers, tablefmt="pretty"))
