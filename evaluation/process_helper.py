import subprocess
import enum
import json


class ConfigKey(enum.Enum):
    ORG_COUNT = 1
    TRANSACTION_COUNT = 2
    ADDRESSES_PER_ORGANIZATION = 3


def run_benchmark_and_get_proof_time() -> str:
    result = subprocess.run(
        ["cargo", "run", "--example", "ot", "--release"], stdout=subprocess.PIPE
    )
    out = result.stdout.decode("utf-8").split("\n")
    return out[len(out) - 2]


def extract_time(nova_string: str) -> float:
    # Use a regular expression to find the time value
    prefix = "Nova::prove_step: "
    print(nova_string[len(prefix) : -1])
    return float(nova_string[len(prefix) : -1])


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
