cfg_if::cfg_if! {
    if #[cfg(not(target_arch = "wasm32"))] {

        #[allow(clippy::module_inception)]
        pub mod nexus;

        pub mod error;
        pub mod args;
        pub mod context;
        pub mod imports;
        pub mod event;
        pub mod result;
        pub mod utils;
        pub mod config;
        pub mod connection;
        pub mod delegate;
        pub mod group;
        pub mod node;
        pub mod services;
        pub mod tpl;
        pub mod transport;
        pub mod monitor;
        pub mod rpc;

        pub mod prelude {
            pub use crate::nexus::Nexus;
        }
    }
}
