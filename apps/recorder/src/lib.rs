#![feature(
    duration_constructors_lite,
    assert_matches,
    unboxed_closures,
    impl_trait_in_bindings,
    iterator_try_collect,
    async_fn_traits,
    error_generic_member_access,
    associated_type_defaults,
    impl_trait_in_fn_trait_return
)]
#![allow(clippy::enum_variant_names)]
pub use downloader;
pub mod app;
pub mod auth;
pub mod cache;
pub mod crypto;
pub mod database;
pub mod errors;
pub mod extract;
pub mod graphql;
pub mod logger;
pub mod media;
pub mod message;
pub mod migrations;
pub mod models;
pub mod storage;
pub mod task;
pub mod utils;
pub mod web;

#[cfg(any(test, feature = "test-utils"))]
pub mod test_utils;
