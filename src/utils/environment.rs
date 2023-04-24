use std::env;

pub fn get_env(s: &str) -> String {
    return env::var(s).expect(format!("Could not find {} environment variable", s).as_str());
}
