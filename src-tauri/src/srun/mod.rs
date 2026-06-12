//! 深澜校园网认证协议：Challenge → XEncode → HMAC-MD5 → SHA1

mod base64;
mod xencode;
mod auth;

pub use auth::{do_login, check_online, get_ip};
