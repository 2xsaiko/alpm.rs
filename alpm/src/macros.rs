#[macro_export]
macro_rules! set_logcb {
    ( $handle:ident, $f:ident ) => {{
        use std::ffi::{c_void, CStr};
        use std::os::raw::{c_char, c_int};
        use std::ptr;

        extern "C" {
            fn vasprintf(
                str: *const *mut c_char,
                fmt: *const c_char,
                args: *mut __va_list_tag,
            ) -> c_int;
            fn free(ptr: *mut c_void);
        }

        unsafe extern "C" fn c_logcb(
            level: alpm_loglevel_t,
            fmt: *const c_char,
            args: *mut __va_list_tag,
        ) {
            let buff = ptr::null_mut();
            let n = vasprintf(&buff, fmt, args);
            if n != -1 {
                let s = CStr::from_ptr(buff);
                let level = LogLevel::from_bits(level).unwrap();
                $f(level, &s.to_string_lossy());
                free(buff as *mut c_void);
            }
        }

        unsafe { alpm_option_set_logcb($handle.handle, Some(c_logcb)) };
    }};
}

#[macro_export]
macro_rules! set_totaldlcb {
    ( $handle:ident, $f:ident ) => {{
        unsafe extern "C" fn c_dlcb(
            filename: *const,
            xfered: off_t,
            total: off_t,
        ) {
                let filename = CStr::from_ptr(filename);
                let filename = filename.to_str().unwrap();
                $f(&filename, xfered as u64, total as u64);
        }

        unsafe { alpm_option_set_logcb($handle.handle, Some(c_dlcb)) };
    }};
}

#[macro_export]
macro_rules! set_fetchcb {
    ( $handle:ident, $f:ident ) => {{
        use crate::FetchCBReturn;
        use std::ffi::CStr;
        use std::os::raw::{c_char, c_int};

        unsafe extern "C" fn c_fetchcb(
            url: *const c_char,
            localpath: *const c_char,
            force: c_int,
        ) -> c_int {
            let url = CStr::from_ptr(url).to_str().unwrap();
            let localpath = CStr::from_ptr(localpath).to_str().unwrap();
            let ret = $f(url, localpath, force != 0);

            match ret {
                FetchCBReturn::Ok => 0,
                FetchCBReturn::Err => -1,
                FetchCBReturn::FileExists => 1,
            }
        }

        unsafe { alpm_option_set_fetchcb($handle.handle, Some(c_fetchcb)) };
    }};
}

#[macro_export]
macro_rules! set_dlcb {
    ( $handle:ident, $f:ident ) => {{
        unsafe extern "C" fn c_totaldlcb(total: off_t) {
            $f(total as u64);
        }

        unsafe { alpm_option_set_logcb($handle.handle, Some(c_totaldlcb)) };
    }};
}

#[macro_export]
macro_rules! set_eventcb {
    ( $handle:ident, $f:ident ) => {{
        use std::ptr;

        static mut C_ALPM_HANDLE: *mut alpm_handle_t = ptr::null_mut();
        unsafe {
            C_ALPM_HANDLE = $handle.handle;
        }

        unsafe extern "C" fn c_eventcb(event: *mut alpm_event_t) {
            let event = Event::new(C_ALPM_HANDLE, event);
            $f(event);
        }

        unsafe { alpm_option_set_eventcb($handle.handle, Some(c_eventcb)) };
    }};
}

#[macro_export]
macro_rules! set_questioncb {
    ( $handle:ident, $f:ident ) => {{
        use std::ptr;

        static mut C_ALPM_HANDLE: *mut alpm_handle_t = ptr::null_mut();
        unsafe {
            C_ALPM_HANDLE = $handle.handle;
        }

        unsafe extern "C" fn c_questioncb(question: *mut alpm_question_t) {
            let question = Question::new(C_ALPM_HANDLE, question);
            $f(question);
        }

        unsafe { alpm_option_set_questioncb($handle.handle, Some(c_questioncb)) };
    }};
}

#[macro_export]
macro_rules! set_progresscb {
    ( $handle:ident, $f:ident ) => {{
        use std::ffi::CStr;
        use std::mem::transmute;
        use std::os::raw::c_char;

        unsafe extern "C" fn c_progresscb(
            progress: alpm_progress_t,
            pkgname: *const c_char,
            percent: c_int,
            howmany: usize,
            current: usize,
        ) {
            let pkgname = CStr::from_ptr(pkgname);
            let pkgname = pkgname.to_str().unwrap();
            let progress = transmute::<alpm_progress_t, Progress>(progress);
            $f(progress, &pkgname, percent as i32, howmany, current);
        }

        unsafe { alpm_option_set_progresscb($handle.handle, Some(c_progresscb)) };
    }};
}

#[macro_export]
macro_rules! log_action {
    ($handle:ident, $prefix:tt, $($arg:tt)*) => ({
        let mut s = format!($($arg)*);
        s.push('\n');
        let s = CString::new(s).unwrap();
        let p = CString::new($prefix).unwrap();

        let ret = unsafe { alpm_logaction($handle.handle, p.as_ptr(), s.as_ptr()) };
        $handle.check_ret(ret)
    })
}
