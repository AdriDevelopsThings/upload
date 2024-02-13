use rand::{distributions::Alphanumeric, Rng};

/// generate a string of random alphanumeric characters
/// the string will have the length `length`
pub fn generate_random_characters(length: usize) -> String {
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(length)
        .map(char::from)
        .collect()
}
