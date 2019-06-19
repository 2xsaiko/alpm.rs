use crate::{free, Alpm, Conflict, Db, DepMissing, Depend, FileConflict, Group, Package};

use std::ffi::{c_void, CStr};
use std::iter::Iterator;
use std::marker::PhantomData;
use std::os::raw::c_char;

use alpm_sys::*;

macro_rules! size_hint {
    ( ) => {
        fn size_hint(&self) -> (usize, Option<usize>) {
            use alpm_sys::*;
            let size = unsafe { alpm_list_count(self.current) };
            (size, Some(size))
        }
    };
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum FreeMethod {
    FreeList,
    FreeInner,
    FreeConflict,
    FreeFileConflict,
    FreeDepMissing,
    None,
}

#[derive(Debug)]
pub struct AlpmList<'a, T> {
    pub(crate) handle: &'a Alpm,
    pub(crate) list: *mut alpm_list_t,
    pub(crate) current: *mut alpm_list_t,
    pub(crate) free: FreeMethod,
    pub(crate) _marker: PhantomData<T>,
}

impl<'a, T> AlpmList<'a, T> {
    pub(crate) fn new(
        handle: &'a Alpm,
        list: *mut alpm_list_t,
        free: FreeMethod,
    ) -> AlpmList<'a, T> {
        AlpmList {
            handle,
            list,
            current: list,
            free,
            _marker: PhantomData,
        }
    }

    pub fn iter(&self) -> AlpmList<'a, T> {
        AlpmList {
            handle: self.handle,
            list: self.list,
            current: self.current,
            free: FreeMethod::None,
            _marker: self._marker,
        }
    }
}

impl<'a> Iterator for AlpmList<'a, Package<'a>> {
    type Item = Package<'a>;
    size_hint!();

    fn next(&mut self) -> Option<Self::Item> {
        unsafe {
            if self.current.is_null() {
                None
            } else {
                let data = (*(self.current)).data;
                let data = data as *mut alpm_pkg_t;
                self.current = alpm_list_next(self.current);
                let pkg = Package {
                    pkg: data,
                    handle: self.handle,
                    drop: false,
                };
                Some(pkg)
            }
        }
    }

}

impl<'a> Iterator for AlpmList<'a, Group<'a>> {
    type Item = Group<'a>;
    size_hint!();

    fn next(&mut self) -> Option<Self::Item> {
        unsafe {
            if self.current.is_null() {
                None
            } else {
                let data = (*(self.current)).data;
                let data = data as *mut alpm_group_t;
                self.current = alpm_list_next(self.current);
                let group = Group {
                    handle: self.handle,
                    inner: data,
                };
                Some(group)
            }
        }
    }
}

impl<'a> Iterator for AlpmList<'a, Depend<'a>> {
    type Item = Depend<'a>;
    size_hint!();

    fn next(&mut self) -> Option<Self::Item> {
        unsafe {
            if self.current.is_null() {
                None
            } else {
                let data = (*(self.current)).data;
                let data = data as *mut alpm_depend_t;
                self.current = alpm_list_next(self.current);
                let pkg = Depend {
                    inner: data,
                    drop: false,
                    phantom: PhantomData,
                };
                Some(pkg)
            }
        }
    }
}

impl<'a> Iterator for AlpmList<'a, FileConflict> {
    type Item = FileConflict;
    size_hint!();

    fn next(&mut self) -> Option<Self::Item> {
        unsafe {
            if self.current.is_null() {
                None
            } else {
                let data = (*(self.current)).data;
                let data = data as *mut alpm_fileconflict_t;
                self.current = alpm_list_next(self.current);
                let pkg = FileConflict { inner: data };
                Some(pkg)
            }
        }
    }
}

impl<'a> Iterator for AlpmList<'a, DepMissing> {
    type Item = DepMissing;
    size_hint!();

    fn next(&mut self) -> Option<Self::Item> {
        unsafe {
            if self.current.is_null() {
                None
            } else {
                let data = (*(self.current)).data;
                let data = data as *mut alpm_depmissing_t;
                self.current = alpm_list_next(self.current);
                let pkg = DepMissing { inner: data };
                Some(pkg)
            }
        }
    }
}

impl<'a> Iterator for AlpmList<'a, Conflict> {
    type Item = Conflict;
    size_hint!();

    fn next(&mut self) -> Option<Self::Item> {
        let drop = self.free != FreeMethod::FreeList && self.free != FreeMethod::None;

        unsafe {
            if self.current.is_null() {
                None
            } else {
                let data = (*(self.current)).data;
                let data = data as *mut alpm_conflict_t;
                self.current = alpm_list_next(self.current);
                let pkg = Conflict {
                    inner: data,
                    drop,
                };
                Some(pkg)
            }
        }
    }
}

impl<'a> Iterator for AlpmList<'a, Db<'a>> {
    type Item = Db<'a>;
    size_hint!();

    fn next(&mut self) -> Option<Self::Item> {
        unsafe {
            if self.current.is_null() {
                None
            } else {
                let data = (*(self.current)).data;
                let data = data as *mut alpm_db_t;
                self.current = alpm_list_next(self.current);
                let pkg = Db {
                    db: data,
                    handle: self.handle,
                };
                Some(pkg)
            }
        }
    }
}

impl<'a> Iterator for AlpmList<'a, &'a str> {
    type Item = &'a str;
    size_hint!();

    fn next(&mut self) -> Option<Self::Item> {
        unsafe {
            if self.current.is_null() {
                None
            } else {
                let data = (*(self.current)).data;
                let data = data as *const c_char;
                self.current = alpm_list_next(self.current);
                let s = CStr::from_ptr(data);
                Some(s.to_str().unwrap())
            }
        }
    }
}

impl<'a> Iterator for AlpmList<'a, String> {
    type Item = String;
    size_hint!();

    fn next(&mut self) -> Option<Self::Item> {
        unsafe {
            if self.current.is_null() {
                None
            } else {
                let data = (*(self.current)).data;
                let data = data as *const c_char;
                self.current = alpm_list_next(self.current);
                let s = CStr::from_ptr(data);
                Some(s.to_str().unwrap().into())
            }
        }
    }
}

impl<'a> IntoIterator for &'a AlpmList<'a, Package<'a>> {
    type Item = Package<'a>;
    type IntoIter = AlpmList<'a, Package<'a>>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

unsafe extern "C" fn fileconflict_free(ptr: *mut c_void) {
    alpm_fileconflict_free(ptr as *mut alpm_fileconflict_t);
}

unsafe extern "C" fn depmissing_free(ptr: *mut c_void) {
    alpm_depmissing_free(ptr as *mut alpm_depmissing_t);
}

unsafe extern "C" fn conflict_free(ptr: *mut c_void) {
    alpm_conflict_free(ptr as *mut alpm_conflict_t);
}

impl<'a, T> Drop for AlpmList<'a, T> {
    fn drop(&mut self) {
        match self.free {
            FreeMethod::None => {}
            FreeMethod::FreeList => {
                unsafe { alpm_list_free(self.list) };
            }
            FreeMethod::FreeInner => {
                unsafe { alpm_list_free_inner(self.list, Some(free)) };
                unsafe { alpm_list_free(self.list) };
            }
            FreeMethod::FreeConflict => {
                unsafe { alpm_list_free_inner(self.list, Some(conflict_free)) };
                unsafe { alpm_list_free(self.current) };
            }
            FreeMethod::FreeFileConflict => {
                unsafe { alpm_list_free_inner(self.list, Some(fileconflict_free)) };
                unsafe { alpm_list_free(self.current) };
            }
            FreeMethod::FreeDepMissing => {
                unsafe { alpm_list_free_inner(self.list, Some(depmissing_free)) };
                unsafe { alpm_list_free(self.current) };
            }
        }
    }
}
