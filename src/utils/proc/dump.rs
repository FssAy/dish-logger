use crate::winapi::*;
use sysinfo::{SystemExt, ProcessExt, System, AsU32};


#[derive(Debug, Clone)]
pub struct DumpProc {
    pub parent: DWORD,
    pub all: Vec<DWORD>
}

impl From<&str> for DumpProc {
    fn from(proc_name: &str) -> Self {
        let mut sys = System::new();
        sys.refresh_processes();

        let mut parent = 0;
        let mut all = Vec::new();
        for (pid, proc) in sys.processes() {
            if proc.name() == proc_name {
                if parent == 0 {
                    if sys.process(proc.parent().unwrap_or_default()).is_none() {
                        parent = pid.as_u32();
                    }
                }

                all.push(pid.as_u32());
            }
        }

        Self {
            parent,
            all,
        }
    }
}
