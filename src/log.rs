// SPDX-License-Identifier: MIT

use std::cell::RefCell;
use std::fmt;
use std::io::Write;

use libc::{c_int, LOG_ERR};

pub(crate) fn syslog(level: c_int, args: fmt::Arguments<'_>) {
    thread_local!(static MSG_BUFFER: RefCell<Vec<u8>> = RefCell::new(Vec::with_capacity(512)));

    MSG_BUFFER.with(|msg_buffer| {
        let mut msg_buffer = msg_buffer.borrow_mut();

        msg_buffer.clear();

        match msg_buffer.write_fmt(args) {
            Ok(_) => {
                msg_buffer.push(b'\0');

                unsafe { libc::syslog(level, b"%s\0".as_ptr() as _, msg_buffer.as_ptr()) };
            }
            Err(e) => {
                msg_buffer.clear();

                let _ = msg_buffer.write_all(b"Failed to format message: ");

                if write!(&mut msg_buffer, "{}", e).is_err() {
                    msg_buffer.clear();

                    let _ = msg_buffer.write_all(b"Failed to format message and error message");
                }

                msg_buffer.push(b'\0');

                unsafe { libc::syslog(LOG_ERR, b"%s\0".as_ptr() as _, msg_buffer.as_ptr()) }
            }
        }
    });
}

macro_rules! error {
    ($($arg:tt)+) => {
        $crate::log::syslog(::libc::LOG_ERR, format_args!($($arg)+))
    };
}
macro_rules! info {
    ($($arg:tt)+) => {
        $crate::log::syslog(::libc::LOG_NOTICE, format_args!($($arg)+))
    };
}

pub(crate) use error;
pub(crate) use info;
