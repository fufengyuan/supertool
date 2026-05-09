/// UDS Server 模块 — CLI (stool) 的 JSON-over-Unix-Socket 服务端
mod protocol;
mod router;
mod server;

pub use server::{resolve_socket_path, UdsServer};
