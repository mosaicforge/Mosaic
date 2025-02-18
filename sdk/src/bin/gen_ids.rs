use sdk::{self, ids};
use std::env::args;

fn main() {
    let num_ids = args()
        .nth(1)
        .unwrap_or("8".to_string())
        .parse::<u32>()
        .unwrap();

    for _ in 0..num_ids {
        println!("{}", ids::create_geo_id());
    }
}
