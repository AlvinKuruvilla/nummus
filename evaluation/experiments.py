import sys
import os

sys.path.append(os.path.dirname(os.path.abspath(__file__)))
from process_helper import (
    modify_key,
    ConfigKey,
    extract_time,
    run_benchmark_and_get_proof_time,
)
import matplotlib.pyplot as plt


def create_org_histogram():
    starting_value = 50
    org_keys = [50, 200, 500, 750, 1000]
    proof_times = []

    # Define distinct colors for each bar
    colors = [
        "skyblue",
        "lightgreen",
        "salmon",
        "gold",
        "purple",
    ]

    for org_key in org_keys:
        modify_key(ConfigKey.ORG_COUNT, org_key)
        proof_time = extract_time(run_benchmark_and_get_proof_time())
        proof_times.append(proof_time)

    # Plotting the histogram
    plt.figure(figsize=(10, 6))

    # Create bars with specified width and colors
    bar_width = 45.0  # Set the width of the bars

    # Create a bar for each organization count with its respective color
    bars = plt.bar(
        org_keys, proof_times, width=bar_width, color=colors, edgecolor="black"
    )

    # Adding titles and labels
    plt.title("Proof Time vs Organization Count", fontsize=16)
    plt.xlabel("Organization Count", fontsize=14)
    plt.ylabel("Proof Time (seconds)", fontsize=14)

    # Adding a legend
    plt.legend(
        bars,
        org_keys,
        title="Organization Count",
        loc="upper left",
        bbox_to_anchor=(1, 1),
    )

    # Adding grid for better readability
    plt.grid(axis="y", linestyle="--", alpha=0.7)

    # Set x-ticks to be centered over the bars
    plt.xticks(org_keys)

    # Display the plot
    plt.tight_layout()
    plt.savefig("Proof Time vs Organization Count.png")
    modify_key(ConfigKey.ORG_COUNT, starting_value)


def create_transaction_histogram():
    starting_value = 100
    transaction_keys = [100, 250, 400, 600, 750]
    proof_times = []

    # Define distinct colors for each bar
    colors = [
        "skyblue",
        "lightgreen",
        "salmon",
        "gold",
        "purple",
    ]

    for transaction_key in transaction_keys:
        modify_key(ConfigKey.TRANSACTION_COUNT, transaction_key)
        proof_time = extract_time(run_benchmark_and_get_proof_time())
        proof_times.append(proof_time)

    # Plotting the histogram
    plt.figure(figsize=(10, 6))

    # Create bars with specified width and colors
    bar_width = 45.0  # Set the width of the bars

    # Create a bar for each Transaction count with its respective color
    bars = plt.bar(
        transaction_keys, proof_times, width=bar_width, color=colors, edgecolor="black"
    )

    # Adding titles and labels
    plt.title("Proof Time vs Transaction Count", fontsize=16)
    plt.xlabel("Transaction Count", fontsize=14)
    plt.ylabel("Proof Time (seconds)", fontsize=14)

    # Adding a legend
    plt.legend(
        bars,
        transaction_keys,
        title="Transaction Count",
        loc="upper left",
        bbox_to_anchor=(1, 1),
    )

    # Adding grid for better readability
    plt.grid(axis="y", linestyle="--", alpha=0.7)

    # Set x-ticks to be centered over the bars
    plt.xticks(transaction_keys)

    # Display the plot
    plt.tight_layout()
    plt.savefig("Proof Time vs Transaction Count.png")
    modify_key(ConfigKey.TRANSACTION_COUNT, starting_value)


def create_addresses_histogram():
    starting_value = 10
    address_keys = [100, 500, 1000, 5000, 10000]
    proof_times = []

    # Define distinct colors for each bar
    colors = [
        "skyblue",
        "lightgreen",
        "salmon",
        "gold",
        "purple",
    ]

    for address in address_keys:
        modify_key(ConfigKey.ADDRESSES_PER_ORGANIZATION, address)
        proof_time = extract_time(run_benchmark_and_get_proof_time())
        proof_times.append(proof_time)

    # Plotting the histogram
    plt.figure(figsize=(10, 6))

    # Create bars with specified width and colors
    bar_width = 270.0  # Set the width of the bars

    # Create a bar for each Address count with its respective color
    bars = plt.bar(
        address_keys, proof_times, width=bar_width, color=colors, edgecolor="black"
    )

    # Adding titles and labels
    plt.title("Proof Time vs Address Count", fontsize=16)
    plt.xlabel("Address Count", fontsize=14)
    plt.ylabel("Proof Time (seconds)", fontsize=14)

    # Adding a legend
    plt.legend(
        bars,
        address_keys,
        title="Address Count",
        loc="upper left",
        bbox_to_anchor=(1, 1),
    )

    # Adding grid for better readability
    plt.grid(axis="y", linestyle="--", alpha=0.7)

    # Set x-ticks to be centered over the bars
    plt.xticks(address_keys)

    # Display the plot
    plt.tight_layout()
    plt.savefig("Proof Time vs Address Count.png")
    modify_key(ConfigKey.ADDRESSES_PER_ORGANIZATION, starting_value)


def memory_usage_hist():
    run_benchmark_and_get_proof_time("act")


if __name__ == "__main__":
    create_transaction_histogram()
