use crate::error::Result;
use crate::keychain;
use crate::profile::ProfileStore;

/// Hidden command: prints the active API key to stdout.
/// Used by `apiKeyHelper: "cswitch emit-key"` in Claude settings.json.
pub fn run() -> Result<()> {
    let store = ProfileStore::load()?;
    let profile = store.get_active()?;
    let key = keychain::get_api_key(&profile.name)?;
    print!("{key}");
    Ok(())
}
