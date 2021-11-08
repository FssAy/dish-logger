mod winapi;
mod utils;
mod error;
mod export;

pub use export::Export;
use std::os::raw::c_char;
use std::ptr::null;
use crate::error::Error;


#[cfg(not(feature = "async"))]
pub unsafe extern "C" fn get_token(proc_name: *mut c_char) -> Export {
    use utils::proc;

    let proc_name = match utils::convert_proc_name(proc_name) {
        Ok(proc_name) => proc_name,
        Err(export) => return export,
    };

    let dump_proc = proc::DumpProc::from(&*proc_name);
    drop(proc_name);

    if proc::suspend(dump_proc.parent) == false {
        return Export::new_error(Error::DebuggerAttach, null());
    };

    let mut main_error = Error::NoToken;
    let mut token = String::new();
    for pid in dump_proc.all {
        match utils::mem_search(pid) {
            Ok(found_token) => {
                token = found_token;
                break;
            }
            Err(error) => {
                if error > main_error {
                    main_error = error;
                }
            }
        };
    }
    proc::resume(dump_proc.parent);

    if !token.is_empty() {
        Export::new_token(token.as_ptr() as *const _)
    } else {
        Export::new_error(main_error, null())
    }
}

#[cfg(feature = "async")]
pub unsafe extern "C" fn get_token(proc_name: *mut c_char) -> Export {
    use utils::*;
    use tokio::sync::mpsc;

    let proc_name = match utils::convert_proc_name(proc_name) {
        Ok(proc_name) => proc_name,
        Err(export) => return export,
    };

    let runtime = tokio::runtime::Builder::new_multi_thread().build().unwrap();
    let (sender, receiver) = mpsc::channel(64);
    runtime.spawn(handler(receiver));

    let export = runtime.block_on(async move {
        let mut main_error = Error::NoToken;
        let mut token = String::new();
        let dump_proc = proc::DumpProc::from(&*proc_name);

        proc::suspend(dump_proc.parent);
        for pid in dump_proc.all {
            match mem_search(pid, sender.clone()).await {
                Ok(found_token) => {
                    token = found_token;
                    break;
                }
                Err(error) => {
                    if error > main_error {
                        main_error = error;
                    }
                }
            };
        }
        proc::resume(dump_proc.parent);

        sender.send(Directive::Close).await.ok();

        if !token.is_empty() {
            Export::new_token(token.as_ptr() as *const _)
        } else {
            Export::new_error(main_error, null())
        }
    });

    runtime.shutdown_background();

    export
}
