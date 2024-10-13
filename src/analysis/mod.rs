use std::{collections::HashMap, mem};

pub fn estimate_hashmap_memory_usage<K, V>(map: &HashMap<K, V>) -> f64 {
    let num_elements = map.len();
    let key_size = mem::size_of::<K>();
    let value_size = mem::size_of::<V>();

    // Rough estimate of the number of buckets
    let num_buckets = (num_elements as f64 / 0.75).ceil() as usize;

    // Calculate memory usage
    let bucket_overhead = mem::size_of::<usize>() * num_buckets; // assuming each bucket is a pointer
    let total_memory_bytes = (key_size + value_size) * num_elements + bucket_overhead;
    total_memory_bytes as f64 / (1024.0 * 1024.0 * 1024.0)
}
pub fn estimate_vec_memory_usage_in_gb<T>(vec: &Vec<T>) -> f64 {
    let element_size = mem::size_of::<T>();

    // Capacity may differ from length if there are reserved elements
    let capacity = vec.capacity();

    // Calculate memory usage
    let total_memory_bytes = (element_size * capacity) + mem::size_of::<Vec<T>>();

    // Convert bytes to gigabytes
    total_memory_bytes as f64 / (1024.0 * 1024.0 * 1024.0)
}
