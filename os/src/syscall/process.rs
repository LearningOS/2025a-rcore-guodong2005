//! Process management syscalls
use crate::config::PAGE_SIZE;
use crate::mm::{translated_byte_buffer, MapPermission, PTEFlags, PageTable, VirtAddr};
use crate::task::{
    change_program_brk, current_syscall_times, current_user_token, exit_current_and_run_next,
    mmap_current, munmap_current, suspend_current_and_run_next,
};
use crate::timer::get_time_us;

#[repr(C)]
#[derive(Debug)]
pub struct TimeVal {
    pub sec: usize,
    pub usec: usize,
}

/// task exits and submit an exit code
pub fn sys_exit(_exit_code: i32) -> ! {
    trace!("kernel: sys_exit");
    exit_current_and_run_next();
    panic!("Unreachable in sys_exit!");
}

/// current task gives up resources for other tasks
pub fn sys_yield() -> isize {
    trace!("kernel: sys_yield");
    suspend_current_and_run_next();
    0
}

/// YOUR JOB: get time with second and microsecond
/// HINT: You might reimplement it with virtual memory management.
/// HINT: What if [`TimeVal`] is splitted by two pages ?
pub fn sys_get_time(ts: *mut TimeVal, _tz: usize) -> isize {
    trace!("kernel: sys_get_time");
    let us = get_time_us();
    let time_val = TimeVal {
        sec: us / 1_000_000,
        usec: us % 1_000_000,
    };
    let len = core::mem::size_of::<TimeVal>();
    let token = current_user_token();
    let mut buffers = translated_byte_buffer(token, ts as *const u8, len);
    let src =
        unsafe { core::slice::from_raw_parts((&time_val as *const TimeVal) as *const u8, len) };
    let mut copied = 0;
    for buffer in buffers.iter_mut() {
        let end = copied + buffer.len();
        buffer.copy_from_slice(&src[copied..end]);
        copied = end;
    }
    0
}

/// TODO: Finish sys_trace to pass testcases
/// HINT: You might reimplement it with virtual memory management.
pub fn sys_trace(trace_request: usize, id: usize, data: usize) -> isize {
    trace!("kernel: sys_trace");
    match trace_request {
        0 | 1 => {
            let va = VirtAddr::from(id);
            let vpn = va.floor();
            let page_offset = va.page_offset();
            let page_table = PageTable::from_token(current_user_token());
            let pte = match page_table.translate(vpn) {
                Some(pte) => pte,
                None => return -1,
            };
            let flags = pte.flags();
            if !flags.contains(PTEFlags::U) {
                return -1;
            }
            if trace_request == 0 && !flags.contains(PTEFlags::R) {
                return -1;
            }
            if trace_request == 1 && !flags.contains(PTEFlags::W) {
                return -1;
            }
            let bytes = pte.ppn().get_bytes_array();
            if trace_request == 0 {
                bytes[page_offset] as isize
            } else {
                bytes[page_offset] = data as u8;
                0
            }
        }
        2 => current_syscall_times(id),
        _ => -1,
    }
}

// YOUR JOB: Implement mmap.
pub fn sys_mmap(start: usize, len: usize, prot: usize) -> isize {
    trace!("kernel: sys_mmap");
    let start_va = VirtAddr::from(start);
    if !start_va.aligned() {
        return -1;
    }
    if prot & !0x7 != 0 || prot & 0x7 == 0 {
        return -1;
    }
    let len = if len == 0 {
        0
    } else {
        (len - 1 + PAGE_SIZE) / PAGE_SIZE * PAGE_SIZE
    };
    let mut permission = MapPermission::U;
    if prot & 0x1 != 0 {
        permission |= MapPermission::R;
    }
    if prot & 0x2 != 0 {
        permission |= MapPermission::W;
    }
    if prot & 0x4 != 0 {
        permission |= MapPermission::X;
    }
    if mmap_current(start, len, permission) {
        0
    } else {
        -1
    }
}

// YOUR JOB: Implement munmap.
pub fn sys_munmap(start: usize, len: usize) -> isize {
    trace!("kernel: sys_munmap");
    let start_va = VirtAddr::from(start);
    if !start_va.aligned() {
        return -1;
    }
    let len = if len == 0 {
        0
    } else {
        (len - 1 + PAGE_SIZE) / PAGE_SIZE * PAGE_SIZE
    };
    if munmap_current(start, len) {
        0
    } else {
        -1
    }
}
/// change data segment size
pub fn sys_sbrk(size: i32) -> isize {
    trace!("kernel: sys_sbrk");
    if let Some(old_brk) = change_program_brk(size) {
        old_brk as isize
    } else {
        -1
    }
}
