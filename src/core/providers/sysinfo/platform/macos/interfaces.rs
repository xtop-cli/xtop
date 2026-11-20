//! macOS interface IP probe via `getifaddrs`.
//!
//! Unlike the linux probe (IPv6 only), macOS reports both IPv4 and IPv6
//! addresses per interface. The FFI surface is small and stable:
//! `getifaddrs`/`freeifaddrs`/`inet_ntop` live in libSystem.

use std::collections::HashMap;
use std::ffi::CStr;
use std::os::raw::{c_char, c_int, c_uint, c_void};

const AF_INET: u8 = 2;
const AF_INET6: u8 = 30;

#[repr(C)]
struct Sockaddr {
    sa_len: u8,
    sa_family: u8,
    // Enough room for sockaddr_in6 (28 bytes on 64-bit macOS).
    sa_data: [u8; 26],
}

#[repr(C)]
struct Ifaddrs {
    ifa_next: *mut Ifaddrs,
    ifa_name: *mut c_char,
    ifa_flags: c_uint,
    ifa_addr: *mut Sockaddr,
    ifa_netmask: *mut Sockaddr,
    ifa_datalink: *mut Sockaddr,
    ifa_data: *mut c_void,
}

extern "C" {
    fn getifaddrs(ifap: *mut *mut Ifaddrs) -> c_int;
    fn freeifaddrs(ifa: *mut Ifaddrs);
    fn inet_ntop(af: c_int, src: *const c_void, dst: *mut c_char, size: usize) -> *mut c_char;
}

/// IP address strings per interface name (IPv4 + IPv6).
pub fn read_interface_ips() -> HashMap<String, Vec<String>> {
    let mut map: HashMap<String, Vec<String>> = HashMap::new();
    let mut head: *mut Ifaddrs = std::ptr::null_mut();
    if unsafe { getifaddrs(&mut head) } != 0 || head.is_null() {
        return map;
    }

    let mut seen: Vec<usize> = Vec::new();
    let mut current = head;
    while !current.is_null() && !seen.contains(&(current as usize)) {
        seen.push(current as usize);
        let ifa = unsafe { &*current };
        if !ifa.ifa_name.is_null() && !ifa.ifa_addr.is_null() {
            let sock = unsafe { &*ifa.ifa_addr };
            let name = unsafe { CStr::from_ptr(ifa.ifa_name) }
                .to_string_lossy()
                .into_owned();
            let address = match sock.sa_family {
                AF_INET => format_addr(AF_INET as c_int, &sock.sa_data[2..6]),
                AF_INET6 => format_addr(AF_INET6 as c_int, &sock.sa_data[6..22]),
                _ => None,
            };
            if let Some(addr) = address {
                map.entry(name).or_default().push(addr);
            }
        }
        current = ifa.ifa_next;
    }
    unsafe { freeifaddrs(head) };
    map
}

fn format_addr(family: c_int, bytes: &[u8]) -> Option<String> {
    // `bytes` already slices the family-specific address out of the
    // sockaddr: sockaddr_in holds sin_addr at offset 4 (sa_data[2..6]) and
    // sockaddr_in6 holds sin6_addr at offset 8 (sa_data[6..22]).
    let src = bytes.as_ptr() as *const c_void;
    let mut buf = [0 as c_char; 46]; // INET6_ADDRSTRLEN
    let res = unsafe { inet_ntop(family, src, buf.as_mut_ptr(), buf.len()) };
    if res.is_null() {
        return None;
    }
    unsafe { CStr::from_ptr(buf.as_ptr()) }
        .to_str()
        .ok()
        .map(str::to_string)
}
