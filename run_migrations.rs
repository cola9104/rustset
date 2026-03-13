// Standalone migration runner
// Compile with: rustc --edition 2021 run_migrations.rs -L target/debug/deps --extern sea_orm --extern sea_orm_migration --extern backend

use sea_orm_migration::prelude::*;

fn main() {
    println!("This script needs to be compiled and run with the backend library.");
    println!("Please run: cargo run --bin backend");
    println!("The backend will automatically run migrations on startup.");
}
