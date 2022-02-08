#![allow(non_snake_case)]
#![allow(dead_code)]
pub mod api;
pub mod consts;
pub mod discord;
pub mod third_party;

#[macro_use] extern crate lazy_static;
#[macro_use] extern crate magic_crypt;

use discord::discord::run;

#[tokio::main]
async fn main() {
    run().await;
}
