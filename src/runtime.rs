use std::cell::SyncUnsafeCell;

static mut SOURCE: SyncUnsafeCell<String> = SyncUnsafeCell::new(String::new());

pub fn set_source(source: impl AsRef<str>) {
    unsafe { *SOURCE.get_mut() = source.as_ref().into() };
}

pub fn append_source(src: impl AsRef<str>) -> usize {
    let orig = source();
    if orig.is_empty() {
        set_source(src);
        0
    } else {
        set_source(format!("{}\n{}", orig, src.as_ref()));
        orig.len() + 1
    }
}

/// Reference to the currently processed source text.
pub fn source() -> &'static str {
    unsafe { &*SOURCE.get() }
}
