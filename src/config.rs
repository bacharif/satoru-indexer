use std::env;

pub fn get_database_url() -> String {
    env::var("DATABASE_URL").expect("DATABASE_URL must be set")
}

pub fn get_provider_url() -> String {
    "https://ancient-skilled-asphalt.strk-sepolia.quiknode.pro/ab96caa503ba84b2f1631ccf7db3f15380314ed7".to_string()
}
