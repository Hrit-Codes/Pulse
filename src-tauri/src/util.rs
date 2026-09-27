use rand::RngExt;

pub fn generate_pin() -> String {
    let mut rng = rand::rng();
    let pin = rng.random_range(0..=9999);
    format!("{:04}", pin)
}

pub fn check_equal(original_pin:String, new_pin:String) -> bool {
    original_pin == new_pin
}
