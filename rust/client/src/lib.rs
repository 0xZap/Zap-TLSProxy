mod client;
mod http;
mod types;
mod utils;

pub mod prelude {
    pub use crate::client::ZapClient;
    pub use crate::types::{Endpoint, EndpointBuilder, Proof, ZapServerConfig};
}
