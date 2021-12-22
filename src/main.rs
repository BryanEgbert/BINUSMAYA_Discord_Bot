#![allow(non_snake_case)]
mod prelude;
mod discord;

use discord::*;


#[tokio::main]
async fn main() {
    run().await;
}