use std::collections::VecDeque;
use std::sync::{Arc, Condvar, Mutex};

pub struct Sender<T> {
    shared: Arc<Shared<T>>,
}

impl<T> Sender<T> {
    pub fn send(&mut self, t: T) {
        let mut inner = self.shared.inner.lock().unwrap();
        inner.queue.push_back(t);
        // unlock the mutex
        drop(inner);
        // wake receiver waiting on a conditon
        self.shared.not_empty.notify_one();
    }
}

impl<T> Clone for Sender<T> {
    fn clone(&self) -> Self {
        let mut inner = self.shared.inner.lock().unwrap();
        inner.senders += 1;
        drop(inner);
        return Sender {
            // we want to clone the arc not the inner inside the arc
            shared: Arc::clone(&self.shared),
        };
    }
}

impl<T> Drop for Sender<T> {
    fn drop(&mut self) {
        let mut inner = self.shared.inner.lock().unwrap();
        inner.senders -= 1;
        if inner.senders == 0 {
            self.shared.not_empty.notify_one();
        }
    }
}

pub struct Receiver<T> {
    shared: Arc<Shared<T>>,
}

impl<T> Receiver<T> {
    pub fn recv(&mut self) -> Option<T> {
        let mut inner = self.shared.inner.lock().unwrap();
        loop {
            match inner.queue.pop_front() {
                Some(t) => return Some(t),
                None if inner.senders == 0 => return None,
                None => {
                    // wait on the condition
                    // not a spin lock
                    inner = self.shared.not_empty.wait(inner).unwrap();
                }
            }
        }
    }
}

pub struct Inner<T> {
    pub queue: VecDeque<T>,
    pub senders: usize,
}

pub struct Shared<T> {
    pub inner: Mutex<Inner<T>>,
    pub not_empty: Condvar,
}

impl<T> Shared<T> {
    pub fn new() -> Shared<T> {
        return Shared {
            inner: Mutex::new(Inner::new()),
            not_empty: Condvar::new(),
        };
    }
}

impl<T> Inner<T> {
    pub fn new() -> Inner<T> {
        return Inner {
            queue: VecDeque::new(),
            senders: 1,
        };
    }
}

pub fn channel<T>() -> (Sender<T>, Receiver<T>) {
    let shared: Shared<T> = Shared::new();
    let shared = Arc::new(shared);
    return (
        Sender {
            shared: shared.clone(),
        },
        Receiver {
            shared: shared.clone(),
        },
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ping_pong() {
        let (mut tx, mut rx): (Sender<usize>, Receiver<usize>) = channel();
        tx.send(7);
        assert_eq!(Some(7), rx.recv());
    }

    #[test]
    fn closed() {
        let (tx, mut rx) = channel::<()>();
        drop(tx);
        assert_eq!(rx.recv(), None);
    }
}
