mod dump;

pub use dump::DumpProc;
use crate::winapi::*;


pub unsafe fn suspend(pid: DWORD) -> bool {
    if DebugActiveProcess(pid) == FALSE {
        return false;
    }

    true
}

pub unsafe fn resume(pid: DWORD) -> bool {
    if DebugActiveProcessStop(pid) == FALSE {
        return false;
    }

    true
}
