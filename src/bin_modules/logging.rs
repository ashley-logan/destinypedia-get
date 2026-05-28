use better_tracing::{Registry, fmt, prelude::*};

pub fn setup_logging() {
    let stdout = fmt::layer().pretty();
    let sub = Registry::default().with(stdout);
    tracing::subscriber::set_global_default(sub).expect("Unable to set global subscriber");
}
