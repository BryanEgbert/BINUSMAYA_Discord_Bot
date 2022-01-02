#![allow(non_snake_case)]
pub mod third_party;
pub mod discord;
pub mod binusmaya;

#[macro_use]
extern crate lazy_static;

use discord::*;


#[tokio::main]
async fn main() {
    run().await;
}