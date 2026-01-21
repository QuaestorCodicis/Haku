use solana_sdk::signature::{Keypair, Signer};

fn main() {
    println!("\n🔐 Solana Wallet Generator");
    println!("==========================\n");

    let keypair = Keypair::new();

    println!("✅ New Wallet Generated!\n");
    println!("Public Address:");
    println!("{}", keypair.pubkey());
    println!("\nPrivate Key (base58):");
    println!("{}", bs58::encode(keypair.to_bytes()).into_string());

    println!("\n⚠️  IMPORTANT:");
    println!("1. SAVE the private key securely");
    println!("2. Add it to .env as:");
    println!("   WALLET_PRIVATE_KEY={}", bs58::encode(keypair.to_bytes()).into_string());
    println!("3. NEVER share it with anyone");
    println!("4. Fund it with SOL before trading\n");

    // Also save to file (encrypted version)
    println!("💡 TIP: You can also save this to a JSON file:");
    println!("solana-keygen new --outfile ~/trading-wallet.json\n");
}
