#[macro_export] macro_rules! try_lock {
    ($x:expr) => (match $x.try_lock() {
        ::std::result::Result::Ok(guard) => guard,
        _ => return,
    });
}

#[macro_export] macro_rules! try_lock_or_return {
    ($x:expr, $retval:expr) => (match $x.try_lock() {
        ::std::result::Result::Ok(guard) => guard,
        _ => return $retval,
    });
}
