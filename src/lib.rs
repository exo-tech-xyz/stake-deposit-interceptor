use solana_program::declare_id;

pub mod error;
pub mod instruction;
pub mod processor;
pub mod macros;
pub mod state;

#[cfg(not(feature = "no-entrypoint"))]
pub mod entrypoint;

declare_id!("5TAiuAh3YGDbwjEruC1ZpXTJWdNDS7Ur7VeqNNiHMmGV");
