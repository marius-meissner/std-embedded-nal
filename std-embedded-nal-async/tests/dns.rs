//! Tests for the DNS resolution parts
//!
//! This depends on a common setup of "localhost" names.

use embedded_nal_async::{Dns, AddrType};

#[test]
fn resolve_localhost() {
    let stack = std_embedded_nal_async::Stack::default();
    async_std::task::block_on(async move {
        let localhost_v4 = stack.get_host_by_name("localhost", AddrType::IPv4).await.unwrap();
        assert!(localhost_v4 == "127.0.0.1".parse::<embedded_nal_async::IpAddr>().unwrap());
        let localhost_v6 = stack.get_host_by_name("localhost", AddrType::IPv6).await.unwrap();
        assert!(localhost_v6 == "::1".parse::<embedded_nal_async::IpAddr>().unwrap());
        let localhost_any = stack.get_host_by_name("localhost", AddrType::Either).await.unwrap();
        assert!(localhost_any == "::1".parse::<embedded_nal_async::IpAddr>().unwrap());
    });
}

#[test]
fn resolve_invalid() {
    let stack = std_embedded_nal_async::Stack::default();
    async_std::task::block_on(async move {
        assert!(stack.get_host_by_name("example.invalid", AddrType::Either).await.is_err());
    });
}

#[test]
fn reverse_localhost() {
    let stack = std_embedded_nal_async::Stack::default();
    async_std::task::block_on(async move {
        let localhost_v4 = stack.get_host_by_address("127.0.0.1".parse().unwrap()).await.unwrap();
        assert!(localhost_v4 == "localhost");
        let localhost_v6 = stack.get_host_by_address("::1".parse().unwrap()).await.unwrap();
        assert!(localhost_v6 == "localhost");
    });
}

#[test]
fn reverse_invalid() {
    let stack = std_embedded_nal_async::Stack::default();
    async_std::task::block_on(async move {
        let broadcast = "255.255.255.255".parse().unwrap();
        assert!(dbg!(stack.get_host_by_address(broadcast).await).is_err());
    });
}
