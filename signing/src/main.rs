use std::{fs::File, io::{Read, Write}};

use secp256k1::{Message, PublicKey, Secp256k1, SecretKey};
use rand::rngs::OsRng;
use sha2::{Sha256, Digest};
use bincode::{serialize, deserialize};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let secp = Secp256k1::new();
    let mut rng = OsRng::default();

    // Generate key pair
    let (secret_key, public_key) = secp.generate_keypair(&mut rng);

    // Message to be signed (this could be your transaction data)
    // let message = "Hello, Blockchain!";

    // Sign the message
    // let signature = sign_message(&secp, &secret_key, message)?;
    // let sig_str = signature.to_string();
    // let sig2 = Signature::from_str(&sig_str).unwrap();
    // println!("Signature: \n{}\n{}",signature, sig2);

    // Derive address from public key (example using a simple hash, actual derivation may vary)
    let address = derive_address(&public_key);

    // Serialize the keys and address
    let private_key_bytes = secret_key[..].to_vec();  // Private key as bytes
    let public_key_bytes = public_key.serialize().to_vec(); // Public key as bytes
    let address_bytes = address.to_vec();  // Address as bytes

    // Combine all bytes into a single structure (tuple)
    let data = (private_key_bytes, public_key_bytes, address_bytes);

    // Open a file to write the data
    let mut file = File::create("keys.bin")?;
    
    // Serialize and write the data to the file
    let encoded: Vec<u8> = serialize(&data).unwrap();
    file.write_all(&encoded)?;

    println!("Keys and address written to keys.bin");

    // // Verify the signature
    // let is_valid = verify_signature(&secp, &public_key, message, &signature)?;
    // println!("Signature is valid: {}", is_valid);

    // Open the file for reading
    let mut file = File::open("keys.bin")?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;

    // Deserialize the data
    let (private_key_bytes, public_key_bytes, address_bytes): (Vec<u8>, Vec<u8>, Vec<u8>) = deserialize(&buffer).unwrap();

    // Reconstruct the keys from bytes
    let secret_key2 = SecretKey::from_slice(&private_key_bytes).expect("32 bytes, within curve order");
    let public_key2 = PublicKey::from_slice(&public_key_bytes).expect("Compressed or uncompressed public key");

    // Print out the data for verification
    println!("Private Key: \n{:?}\n{:?}\n", secret_key.display_secret(), secret_key2.display_secret());
    println!("Private Key: \n{:?}\n{:?}\n", public_key, public_key2);
    println!("Address: \n{:?}\n{:?}\n", address, address_bytes);

    Ok(())
}

fn sign_message(secp: &Secp256k1<secp256k1::All>, secret_key: &SecretKey, message: &str) -> Result<secp256k1::ecdsa::Signature, Box<dyn std::error::Error>> {
    let message = create_message_hash(message);
    let signature = secp.sign_ecdsa(&message, secret_key);
    Ok(signature)
}

fn verify_signature(secp: &Secp256k1<secp256k1::All>, public_key: &PublicKey, message: &str, signature: &secp256k1::ecdsa::Signature) -> Result<bool, Box<dyn std::error::Error>> {
    let message = create_message_hash(message);
    Ok(secp.verify_ecdsa(&message, signature, public_key).is_ok())
}

fn create_message_hash(message: &str) -> Message {
    let mut hasher = Sha256::new();
    hasher.update(message.as_bytes());
    let result = hasher.finalize();
    Message::from_digest_slice(&result).expect("32 bytes")
}

fn derive_address(public_key: &PublicKey) -> [u8; 20] {
    let mut hasher = Sha256::new();
    hasher.update(&public_key.serialize());
    let result = hasher.finalize();
    let mut address = [0u8; 20];
    address.copy_from_slice(&result[12..32]);
    address
}