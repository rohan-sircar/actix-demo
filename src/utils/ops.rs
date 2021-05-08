use std::fmt::Display;

pub trait LogErrorResult<T, E> {
    fn log_err(self) -> Result<T, E>;
}
impl<T, E: Display> LogErrorResult<T, E> for Result<T, E> {
    fn log_err(self) -> Result<T, E> {
        self.map_err(|err| {
            log::error!("{}", err.to_string());
            err
        })
    }
}

trait ResultOps<T, E> {
    fn tap<U, F: FnOnce(T) -> U>(self, op: F) -> Result<T, E>;
    fn tap_err<F, O: FnOnce(E) -> F>(self, op: O) -> Result<T, E>;
}

impl<T: Clone, E: Clone> ResultOps<T, E> for Result<T, E> {
    fn tap<U, F: FnOnce(T) -> U>(self, op: F) -> Result<T, E> {
        self.map(|x| {
            op(x.clone());
            x
        })
    }

    fn tap_err<F, O: FnOnce(E) -> F>(self, op: O) -> Result<T, E> {
        self.map_err(|err| {
            op(err.clone());
            err
        })
    }
}
