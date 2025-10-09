use std::ffi::CString;
use libc::{c_char, c_int, execvp, fork, prctl, wait};

// Constant defined in prctl.h
// See prctl(2) for more details
const PR_SET_CHILD_SUBREAPER: c_int = 36;
const PR_SET_NAME: c_int = 15;

pub struct Args {
    pub command: String,
    pub workdir: String
}

pub fn sub_reaper(args: Args, other: &[String]) -> i32 {
    let mut command: Vec<String> = Vec::with_capacity(1 + other.len());
    command.push(args.command.clone());
    command.extend_from_slice(other);
    let workdir = &args.workdir;
    let mut child_status: i32 = 0;

    let proc_name = b"reaper\0";
    unsafe { prctl(PR_SET_NAME, proc_name.as_ptr() as *const c_char, 0, 0, 0); }
    unsafe { prctl(PR_SET_CHILD_SUBREAPER, 1, 0, 0, 0); }

    let pid = unsafe { fork() };
    if pid == -1 { eprintln!("Fork failed"); std::process::exit(1); }

    if pid == 0 {
        if let Err(e) = std::env::set_current_dir(workdir) { eprintln!("Failed to chdir: {}", e); std::process::exit(1); }

        let cstrings: Vec<CString> = command.iter().map(|s| CString::new(s.as_bytes()).unwrap()).collect();
        let mut c_ptrs: Vec<*const c_char> = cstrings.iter().map(|cs| cs.as_ptr()).collect();
        c_ptrs.push(std::ptr::null());

        unsafe {
            execvp(c_ptrs[0], c_ptrs.as_ptr());
            eprintln!("execvp failed");
            std::process::exit(1);
        }
    }

    loop {
        let child_pid = unsafe { wait(&mut child_status) };
        if child_pid < 0 { break; }
    }
    child_status
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let sep = args.iter().position(|x| x == "--").expect("Missing '--' in arguments");

    let mut argv = args[sep + 1..].to_vec();
    if argv.is_empty() { eprintln!("Usage: {} SteamLaunch -- <command> [args...]", args[0]); std::process::exit(1); }
    let command = argv.remove(0);
    let workdir = std::env::current_dir().expect("Failed to get current directory").to_string_lossy().to_string();
    let args_obj = Args { command, workdir };
    sub_reaper(args_obj, &argv);
}
