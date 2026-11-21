//! macOS process thread count probe via `proc_pidinfo`.
//!
//! The `PROC_PIDTASKINFO` flavor returns a `proc_taskinfo` whose
//! `pti_threadnum` is the number of threads in the task. Layout is stable
//! across modern macOS releases; the call returns 0 when the process is gone
//! or the buffer size does not match.

use std::mem::size_of;
use std::os::raw::{c_int, c_void};

const PROC_PIDTASKINFO: c_int = 4;

#[repr(C)]
#[derive(Clone, Copy)]
struct ProcTaskInfo {
    pti_virtual_size: u64,
    pti_resident_size: u64,
    pti_total_user: u64,
    pti_total_system: u64,
    pti_threads_user: u64,
    pti_threads_system: u64,
    pti_policy: c_int,
    pti_faults: c_int,
    pti_pageins: c_int,
    pti_cow_faults: c_int,
    pti_messages_sent: c_int,
    pti_messages_received: c_int,
    pti_syscalls_mach: c_int,
    pti_syscalls_unix: c_int,
    pti_csw: c_int,
    pti_threadnum: c_int,
    pti_numrunning: c_int,
    pti_priority: c_int,
}

extern "C" {
    fn proc_pidinfo(
        pid: c_int,
        flavor: c_int,
        arg: u64,
        buffer: *mut c_void,
        buffersize: c_int,
    ) -> c_int;
}

/// Thread count of a process, `0` when unavailable.
pub fn read_thread_count(pid: sysinfo::Pid) -> u64 {
    let pid = pid.as_u32() as i32;
    if pid <= 0 {
        return 0;
    }
    let mut info = ProcTaskInfo {
        pti_virtual_size: 0,
        pti_resident_size: 0,
        pti_total_user: 0,
        pti_total_system: 0,
        pti_threads_user: 0,
        pti_threads_system: 0,
        pti_policy: 0,
        pti_faults: 0,
        pti_pageins: 0,
        pti_cow_faults: 0,
        pti_messages_sent: 0,
        pti_messages_received: 0,
        pti_syscalls_mach: 0,
        pti_syscalls_unix: 0,
        pti_csw: 0,
        pti_threadnum: 0,
        pti_numrunning: 0,
        pti_priority: 0,
    };
    let expected = size_of::<ProcTaskInfo>() as c_int;
    let ret = unsafe {
        proc_pidinfo(
            pid,
            PROC_PIDTASKINFO,
            0,
            &mut info as *mut ProcTaskInfo as *mut c_void,
            expected,
        )
    };
    if ret == expected {
        info.pti_threadnum.max(0) as u64
    } else {
        0
    }
}
