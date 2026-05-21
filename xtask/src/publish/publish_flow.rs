use anyhow::{Result, anyhow};
use std::thread::sleep;
use std::time::Duration;

pub(super) fn require_confirmation(yes: bool) -> Result<()> {
    if yes {
        Ok(())
    } else {
        Err(anyhow!(
            "Refusing to publish without --yes. Run `cargo run -p xtask -- publish-plan` first."
        ))
    }
}

pub(super) fn warn_if_registry_token_missing() {
    if std::env::var_os("CARGO_REGISTRY_TOKEN").is_none() {
        println!(
            "Warning: CARGO_REGISTRY_TOKEN is not set; cargo publish will use local cargo credentials if available."
        );
    }
}

pub(super) fn announce_start(crate_count: usize) {
    println!("🚢 Publishing {crate_count} crates to crates.io...");
}

pub(super) fn pause_for_index_propagation(current_index: usize, total: usize, delay_secs: u64) {
    let has_next = current_index
        .checked_add(1)
        .is_some_and(|next| next < total);
    if has_next && delay_secs > 0 {
        println!("Waiting {delay_secs}s for crates.io index propagation before continuing...");
        sleep(Duration::from_secs(delay_secs));
    }
}

pub(super) fn announce_complete() {
    println!("✅ Publish sequence complete!");
}
