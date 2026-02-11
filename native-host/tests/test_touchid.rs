use webtags_host::encryption::EncryptionManager;

#[test]
#[ignore = "Only run manually with: cargo test --test test_touchid -- --ignored"]
fn test_touch_id_integration() {
    // This test will trigger a Touch ID prompt on macOS
    println!("\n🔐 Testing Touch ID integration...");
    println!("You should see a Touch ID prompt appear!\n");

    // Generate and store a key (will trigger Touch ID prompt)
    match EncryptionManager::generate_and_store_key() {
        Ok(()) => {
            println!("✅ Successfully stored encryption key with Touch ID");
            println!("   Check: Did you see a Touch ID prompt?");
        }
        Err(e) => {
            println!("❌ Failed to store key: {e}");
            panic!("Touch ID test failed");
        }
    }

    // Now test encryption (will trigger another Touch ID prompt)
    let manager = EncryptionManager::new(true);
    let test_data = b"Hello, Touch ID!";

    println!("\n🔓 Testing encryption with Touch ID...");
    match manager.encrypt(test_data) {
        Ok(encrypted) => {
            println!("✅ Successfully encrypted data");

            // Test decryption (will trigger Touch ID again)
            println!("\n🔐 Testing decryption with Touch ID...");
            match manager.decrypt(&encrypted) {
                Ok(decrypted) => {
                    println!("✅ Successfully decrypted data");
                    assert_eq!(decrypted, test_data);
                    println!("   Data matches! Touch ID is working correctly! 🎉");
                }
                Err(e) => {
                    println!("❌ Failed to decrypt: {e}");
                    panic!("Decryption failed");
                }
            }
        }
        Err(e) => {
            println!("❌ Failed to encrypt: {e}");
            panic!("Encryption failed");
        }
    }

    // Clean up
    println!("\n🧹 Cleaning up test key...");
    let _ = EncryptionManager::delete_key_from_keychain();
    println!("✅ Test complete!");
}
