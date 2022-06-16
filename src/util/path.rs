/// An extension to `std::path::Path` to reduce boilerplate cote.
pub trait PathExt {
    fn name(&self) -> Option<String>;
    fn ext(&self) -> Option<String>;
}

impl<P: AsRef<std::path::Path>> PathExt for P {
    fn name(&self) -> Option<String> {
        self.as_ref()
            .file_name()
            .map(|name| name.to_string_lossy().into_owned())
    }
    fn ext(&self) -> Option<String> {
        self.as_ref()
            .extension()
            .map(|s| s.to_string_lossy().into_owned())
    }
}
