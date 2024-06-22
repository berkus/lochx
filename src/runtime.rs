use std::cell::SyncUnsafeCell;

static mut SOURCE: SyncUnsafeCell<String> = SyncUnsafeCell::new(String::new());

pub fn set_source(source: String) {
    unsafe { *SOURCE.get_mut() = source };
}

/// Reference to currently processed source text.
pub fn source() -> &'static str {
    unsafe { &*SOURCE.get() }
}
