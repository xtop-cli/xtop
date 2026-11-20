//! `xtop` binary entry point.
//!
//! All argument dispatch lives in the library (`xtop::cli`); this file only
//! forwards the process exit status.

fn main() -> anyhow::Result<()> {
    xtop::cli::run()
}
