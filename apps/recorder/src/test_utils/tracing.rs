use tracing::Level;
use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};
use tracing_tree::HierarchicalLayer;

use crate::logger::MODULE_WHITELIST;

fn build_testing_tracing_filter(level: Level) -> EnvFilter {
    let crate_name = env!("CARGO_PKG_NAME");
    let level = level.as_str().to_lowercase();
    let mut filter = EnvFilter::new(format!("{crate_name}[]={level}"));

    let mut modules = vec!["mockito", "testcontainers"];
    modules.extend(MODULE_WHITELIST.iter());
    for module in modules {
        filter = filter.add_directive(format!("{module}[]={level}").parse().unwrap());
    }

    filter
}

pub fn try_init_testing_tracing(level: Level) {
    let _ = tracing_subscriber::fmt()
        .with_env_filter(build_testing_tracing_filter(level))
        .try_init();
}

pub fn try_init_testing_tracing_only_leaf(level: Level) {
    let _ = tracing_subscriber::registry()
        .with(build_testing_tracing_filter(level))
        .with(
            HierarchicalLayer::new(2)
                .with_targets(true)
                .with_bracketed_fields(true),
        )
        .try_init();
}

/// Set up a thread-local tracing subscriber that captures logs into
/// `tracing_test::internal::global_buf()`.
///
/// Unlike `#[tracing_test::traced_test]` which uses `set_global_default`
/// (panics when another test already set it), this uses
/// `tracing::subscriber::set_default` (thread-local, non-conflicting).
///
/// Returns a `DefaultGuard` that must be held alive for the duration
/// of the test. Pair with [`logs_contain`] to assert on captured logs.
///
/// Tests using this should be annotated with `#[serial]` to avoid
/// concurrent writes to the shared global buffer.
pub fn setup_traced_test() -> tracing::subscriber::DefaultGuard {
    let buf = tracing_test::internal::global_buf();
    buf.lock().unwrap().clear();
    let mock_writer = tracing_test::internal::MockWriter::new(buf);
    let subscriber = tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .with_writer(mock_writer)
        .with_level(true)
        .with_ansi(false)
        .finish();
    tracing::subscriber::set_default(subscriber)
}

/// Check whether the captured logs contain the specified string.
///
/// Must be used together with [`setup_traced_test`].
pub fn logs_contain(val: &str) -> bool {
    let buf = tracing_test::internal::global_buf();
    let logs = String::from_utf8(buf.lock().unwrap().to_vec()).unwrap();
    logs.contains(val)
}
