use std::ffi::{CStr, CString};
use std::os::raw::c_char;

use reaper_medium::{MainThreadScope, ProjectContext, Reaper};

const FIRST_BUFFER: usize = 1024;
const LARGEST_VALUE: usize = 1 << 20;

/// A value in the current project's ExtState; empty and absent are the same to REAPER.
pub(super) fn read(reaper: &Reaper<MainThreadScope>, section: &str, key: &str) -> Option<String> {
    let (section, key) = (CString::new(section).ok()?, CString::new(key).ok()?);
    let project = ProjectContext::CurrentProject.to_raw();
    let mut size = FIRST_BUFFER;
    loop {
        let mut buffer: Vec<c_char> = vec![0; size];
        // SAFETY: the strings and the buffer outlive the call and `size` is the buffer's length.
        let length = unsafe {
            reaper.low().GetProjExtState(
                project,
                section.as_ptr(),
                key.as_ptr(),
                buffer.as_mut_ptr(),
                i32::try_from(size).ok()?,
            )
        };
        let length = usize::try_from(length).ok().filter(|length| *length > 0)?;
        if length >= size {
            size = length
                .checked_add(1)
                .filter(|size| *size <= LARGEST_VALUE)?;
            continue;
        }
        // SAFETY: REAPER wrote a NUL-terminated string into the buffer.
        let value = unsafe { CStr::from_ptr(buffer.as_ptr()) };
        return Some(value.to_string_lossy().into_owned());
    }
}

/// Stores a value; an empty value removes the key. Text with a NUL byte cannot be stored.
pub(super) fn write(reaper: &Reaper<MainThreadScope>, section: &str, key: &str, value: &str) {
    let (Ok(section), Ok(key), Ok(value)) = (
        CString::new(section),
        CString::new(key),
        CString::new(value),
    ) else {
        return;
    };
    // SAFETY: the strings outlive the call.
    unsafe {
        reaper.low().SetProjExtState(
            ProjectContext::CurrentProject.to_raw(),
            section.as_ptr(),
            key.as_ptr(),
            value.as_ptr(),
        );
    }
    reaper.mark_project_dirty(ProjectContext::CurrentProject);
}
