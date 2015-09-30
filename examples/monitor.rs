extern crate libudev;
extern crate libc;

use std::io;
use std::ptr;
use std::thread;

use std::os::unix::io::{AsRawFd};

use libc::{c_void,c_int,c_short,c_ulong,timespec};

#[repr(C)]
struct pollfd {
    fd: c_int,
    events: c_short,
    revents: c_short,
}

#[repr(C)]
struct sigset_t {
    __private: c_void
}

#[allow(non_camel_case_types)]
type nfds_t = c_ulong;

const POLLIN: c_short = 0x0001;

extern "C" {
    fn ppoll(fds: *mut pollfd, nfds: nfds_t, timeout_ts: *mut libc::timespec, sigmask: *const sigset_t) -> c_int;
}

fn main() {
    let context = libudev::Context::new().unwrap();
    monitor(&context).unwrap();
}

fn monitor(context: &libudev::Context) -> io::Result<()> {
    let mut monitor_spec = try!(libudev::MonitorSpec::new(&context));

    try!(monitor_spec.match_subsystem_devtype("usb", "usb_device"));
    let mut monitor = try!(monitor_spec.listen());

    let mut fds = vec!(pollfd { fd: monitor.as_raw_fd(), events: POLLIN, revents: 0 });

    loop {
        let result = unsafe { ppoll((&mut fds[..]).as_mut_ptr(), fds.len() as nfds_t, ptr::null_mut(), ptr::null()) };

        if result < 0 {
            return Err(io::Error::last_os_error());
        }

        let event = match monitor.receive_event() {
            Some(evt) => evt,
            None => {
                thread::sleep_ms(10);
                continue;
            }
        };

        println!("{}: {} {} (subsystem={}, sysname={}, devtype={})",
                 event.sequence_number(),
                 event.event_type(),
                 event.syspath().to_str().unwrap_or("---"),
                 event.subsystem().to_str().unwrap_or(""),
                 event.sysname().to_str().unwrap_or(""),
                 event.devtype().map_or("", |s| { s.to_str().unwrap_or("") }));
    }
}