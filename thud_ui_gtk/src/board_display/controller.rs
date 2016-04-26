macro_rules! try_lock_or_return {
    ($x:expr, $else:expr) =>
        (match $x.try_lock() {
            ::std::result::Result::Ok(guard) => guard,
            _ => return $else,
        });
}

pub mod passive {
}

pub mod interactive {
    pub fn draw(
}
