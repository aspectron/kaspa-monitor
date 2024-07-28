cfg_if::cfg_if! {
    if #[cfg(not(target_arch = "wasm32"))] {

        pub mod connection;
        pub mod error;
        pub mod result;
        pub mod router;
        pub mod server;
        pub mod service;

        pub mod imports;

        pub use service::{WrpcEncoding, WrpcOptions, WrpcService};

    }
}
