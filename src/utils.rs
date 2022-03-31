/// Allows using functions (closures) as methods.
///
/// Can be useful inside the `view!` macro.
pub trait Bind {
    fn bind(&self, f: impl Fn(&Self)) {
        f(self)
    }
}

impl<T> Bind for T {}
