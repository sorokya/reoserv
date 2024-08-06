use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};

pub fn generate_access_token() -> String {
    let mut rng = thread_rng();
    (0..32).map(|_| rng.sample(Alphanumeric) as char).collect()
}
