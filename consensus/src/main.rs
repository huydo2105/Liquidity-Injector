use std::net::SocketAddr;

use ed25519_dalek::SigningKey;
use rand::rngs::OsRng;
use tonic::transport::Server;

use consensus::quorum_proto::{
    collector_service_server::CollectorServiceServer,
    validator_service_server::ValidatorServiceServer,
};
use consensus::collector::CollectorNode;
use consensus::validator::ValidatorNode;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();

    let role = args
        .iter()
        .find(|a| a.starts_with("--role="))
        .map(|a| a.strip_prefix("--role=").unwrap())
        .unwrap_or("collector");

    let port: u16 = args
        .iter()
        .find(|a| a.starts_with("--port="))
        .and_then(|a| a.strip_prefix("--port=").unwrap().parse().ok())
        .unwrap_or(50051);

    let addr: SocketAddr = format!("0.0.0.0:{}", port).parse()?;

    // Generate MRENCLAVE for the expected enclave
    let expected_mrenclave = {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(b"cycles-liquidity-injector-enclave-v0.1.0");
        let result: [u8; 32] = hasher.finalize().into();
        result
    };

    match role {
        "validator" => {
            println!("🔐 Starting Validator Node on {}", addr);
            let signing_key = SigningKey::generate(&mut OsRng);
            println!(
                "   Public Key: {}",
                hex::encode(signing_key.verifying_key().to_bytes())
            );

            let validator = ValidatorNode::new(signing_key, expected_mrenclave);

            Server::builder()
                .add_service(ValidatorServiceServer::new(validator))
                .serve(addr)
                .await?;
        }
        "collector" => {
            println!("📡 Starting Collector Node on {}", addr);

            // In production, load committee from config file.
            // For demo, generate 5 validator keys.
            let mut committee_pubkeys = Vec::new();
            println!("   Demo committee (5 validators):");
            for i in 0..5 {
                let key = SigningKey::generate(&mut OsRng);
                let pk = key.verifying_key().to_bytes();
                println!("   Validator {}: {}", i, hex::encode(pk));
                committee_pubkeys.push(pk);
            }

            // N=5 → f=1 → threshold=3
            let collector = CollectorNode::new(committee_pubkeys, 3);

            Server::builder()
                .add_service(CollectorServiceServer::new(collector))
                .serve(addr)
                .await?;
        }
        _ => {
            eprintln!("Unknown role: {}. Use --role=validator or --role=collector", role);
            std::process::exit(1);
        }
    }

    Ok(())
}
