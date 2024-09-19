use client::prelude::*;

fn main() {
    let client = ZapClient::new(ZapServerConfig::default());
    let endpoint = Endpoint::default();

    let proof = client.prove(endpoint).expect("Failed to prove endpoint");

    println!("{:?}", proof);
}
