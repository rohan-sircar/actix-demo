// use async_recursion::async_recursion;
// use tokio::sync::broadcast;

// pub struct MyBc<T> {
//     sender: broadcast::Sender<T>,
//     receiver: broadcast::Receiver<T>,
// }

// impl<T: Clone> MyBc<T> {
//     #[async_recursion]
//     async fn fib(n: u32) -> u64 {
//         match n {
//             0 => panic!("zero is not a valid argument to fib()!"),
//             1 | 2 => 1,
//             3 => 2,
//             _ => Self::fib(n - 1).await + Self::fib(n - 2).await,
//         }
//     }

//     pub fn new(capacity: usize) -> Self {
//         let (tx, rx) = broadcast::channel(capacity);
//         Self {
//             sender: tx,
//             receiver: rx,
//         }
//     }
// }

// impl<T: Clone> Default for MyBc<T> {
//     fn default() -> Self {
//         Self::new(10)
//     }
// }
