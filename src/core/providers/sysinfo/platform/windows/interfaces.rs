//! Windows interface probe via `GetAdaptersAddresses`.
//!
//! Keyed by the adapter friendly name — the same name sysinfo uses for its
//! network list on Windows (the MIB `Alias` reported by `GetIfTable2`), so
//! the kernel can merge the addresses onto its interfaces by name.

use std::collections::HashMap;

use windows_sys::Win32::Foundation::ERROR_BUFFER_OVERFLOW;
use windows_sys::Win32::NetworkManagement::IpHelper::{
    GetAdaptersAddresses, GAA_FLAG_SKIP_ANYCAST, GAA_FLAG_SKIP_DNS_SERVER, GAA_FLAG_SKIP_MULTICAST,
    IP_ADAPTER_ADDRESSES_LH, IP_ADAPTER_UNICAST_ADDRESS_LH,
};
use windows_sys::Win32::Networking::WinSock::{
    AF_INET, AF_INET6, AF_UNSPEC, SOCKADDR, SOCKADDR_IN, SOCKADDR_IN6,
};

use super::wide_str;

/// Per-interface unicast IPs from `GetAdaptersAddresses`.
pub fn read_interface_ips() -> HashMap<String, Vec<String>> {
    let mut ips = HashMap::new();
    let mut size: u32 = 0;
    let family = AF_UNSPEC.into();
    let flags = GAA_FLAG_SKIP_MULTICAST | GAA_FLAG_SKIP_ANYCAST | GAA_FLAG_SKIP_DNS_SERVER;
    let mut ret = unsafe {
        GetAdaptersAddresses(
            family,
            flags,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            &mut size,
        )
    };
    if ret != ERROR_BUFFER_OVERFLOW {
        return ips;
    }
    let mut buf = vec![0u8; size as usize];
    ret = unsafe {
        GetAdaptersAddresses(
            family,
            flags,
            std::ptr::null_mut(),
            buf.as_mut_ptr() as *mut IP_ADAPTER_ADDRESSES_LH,
            &mut size,
        )
    };
    if ret != 0 {
        return ips;
    }
    let mut adapter = buf.as_ptr() as *const IP_ADAPTER_ADDRESSES_LH;
    while !adapter.is_null() {
        let a = unsafe { &*adapter };
        let name = unsafe { wide_str(a.FriendlyName) };
        let mut addresses = Vec::new();
        let mut unicast = a.FirstUnicastAddress as *const IP_ADAPTER_UNICAST_ADDRESS_LH;
        while !unicast.is_null() {
            let u = unsafe { &*unicast };
            if let Some(ip) = ip_from_sockaddr(u.Address.lpSockaddr) {
                addresses.push(ip);
            }
            unicast = u.Next as *const IP_ADAPTER_UNICAST_ADDRESS_LH;
        }
        if !name.is_empty() && !addresses.is_empty() {
            ips.insert(name, addresses);
        }
        adapter = a.Next as *const IP_ADAPTER_ADDRESSES_LH;
    }
    ips
}

fn ip_from_sockaddr(sockaddr: *const SOCKADDR) -> Option<String> {
    if sockaddr.is_null() {
        return None;
    }
    let sa = unsafe { &*sockaddr };
    match sa.sa_family {
        AF_INET => {
            let v4 = unsafe { &*(sockaddr as *const SOCKADDR_IN) };
            let raw = unsafe { v4.sin_addr.S_un.S_addr };
            let addr = std::net::Ipv4Addr::from(raw.to_ne_bytes());
            Some(addr.to_string())
        }
        AF_INET6 => {
            let v6 = unsafe { &*(sockaddr as *const SOCKADDR_IN6) };
            let addr = std::net::Ipv6Addr::from(unsafe { v6.sin6_addr.u.Byte });
            Some(addr.to_string())
        }
        _ => None,
    }
}
