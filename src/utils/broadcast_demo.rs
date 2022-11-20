// use tokio::sync::broadcast;
use async_recursion::async_recursion;

pub struct MyBc {}

impl MyBc {
    #[async_recursion]
    async fn fib(n: u32) -> u64 {
        match n {
            0 => panic!("zero is not a valid argument to fib()!"),
            1 | 2 => 1,
            3 => 2,
            _ => Self::fib(n - 1).await + Self::fib(n - 2).await,
        }
    }
}
