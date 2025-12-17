use keyring::Entry;

const SERVICE: &str = "zinjvi.typefree";
const ACCOUNT: &str = "openai_api_key";

pub fn save_api_key(key: &str) -> Result<(), keyring::Error> {
    let entry = Entry::new(SERVICE, ACCOUNT)?;

    match entry.set_password(key) {
        Ok(()) => {
            println!("[Keychain] ✅ API key saved successfully to macOS Keychain");
            Ok(())
        }
        Err(e) => {
            eprintln!("[Keychain] ❌ Failed to save API key: {:?}", e);
            Err(e)
        }
    }
}

pub fn load_api_key() -> Result<Option<String>, keyring::Error> {
    println!("[Keychain] Attempting to load API key");
    println!("[Keychain] Service: '{}', Account: '{}'", SERVICE, ACCOUNT);

    let entry = Entry::new(SERVICE, ACCOUNT)?;

    match entry.get_password() {
        Ok(password) => {
            println!("[Keychain] ✅ API key loaded successfully (length: {})", password.len());
            Ok(Some(password))
        }
        Err(keyring::Error::NoEntry) => {
            println!("[Keychain] ℹ️  No API key found in keychain");
            Ok(None)
        }
        Err(e) => {
            eprintln!("[Keychain] ❌ Error loading API key: {:?}", e);
            Err(e)
        }
    }
}

pub fn delete_api_key() -> Result<(), keyring::Error> {
    println!("[Keychain] Attempting to delete API key");

    let entry = Entry::new(SERVICE, ACCOUNT)?;

    match entry.delete_credential() {
        Ok(()) => {
            println!("[Keychain] ✅ API key deleted successfully");
            Ok(())
        }
        Err(keyring::Error::NoEntry) => {
            println!("[Keychain] ℹ️  No API key to delete (not found)");
            Ok(())
        }
        Err(e) => {
            eprintln!("[Keychain] ❌ Error deleting API key: {:?}", e);
            Err(e)
        }
    }
}
