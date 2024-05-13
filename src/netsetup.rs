use core::str::FromStr;
use embassy_net::DhcpConfig;

const HOST_NAME: &str = "picow"; // the name this device will self-identify as

pub(super) const WIFI_NETWORK: &str = "WIFINAME";
pub(super) const WIFI_PASSWORD: &str = "wifipasswd";

pub(super) const SERVER_NAME: &str = "pi2b"; // your server we'll communicate with

// Create a DhcpConfig object that will give a specific hostname to the DHCP
// server while registering.
pub(super) fn dhcp_with_host_name() -> DhcpConfig {
    let mut dhcp = DhcpConfig::default();
    dhcp.hostname = Some(heapless::String::from_str(HOST_NAME).unwrap());
    dhcp
}
