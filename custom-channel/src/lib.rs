use std::collections::VecDeque;
use std::sync::{Arc, Condvar, Mutex};

pub struct Sender<T> {
    inner: Arc<Inner<T>>,
}

impl<T> Sender<T> {
    pub fn send(&mut self, t: T) {
        let mut queue = self.inner.queue.lock().unwrap();
        queue.push_back(t);
        // unlock the mutex
        drop(queue);
        // wake receiver waiting on a conditon
        self.inner.not_empty.notify_one();
    }
}

impl<T> Clone for Sender<T> {
    fn clone(&self) -> Self {
        return Sender {
            // we want to clone the arc not the inner inside the arc
            inner: Arc::clone(&self.inner),
        };
    }
}

pub struct Receiver<T> {
    inner: Arc<Inner<T>>,
}

impl<T> Receiver<T> {
    pub fn recv(&mut self) -> T {
        let mut queue = self.inner.queue.lock().unwrap();
        loop {
            match queue.pop_front() {
                Some(t) => return t,
                None => {
                    // wait on the condition
                    // not a spin lock
                    queue = self.inner.not_empty.wait(queue).unwrap();
                }
            }
        }
    }

    pub fn try_recv(&mut self) -> Option<T> {
        let mut queue = self.inner.queue.lock().unwrap();
        return queue.pop_front();
    }
}

/*pub struct Inner<T> {
    pub queue: Mutex<VecDeque<T>>,
    pub not_empty: Condvar,
}*/

pub struct Inner<T> {
    pub queue: Mutex<VecDeque<T>>,
    pub not_empty: Condvar,
}

impl<T> Inner<T> {
    pub fn new() -> Inner<T> {
        return Inner {
            queue: Mutex::new(VecDeque::new()),
            not_empty: Condvar::new(),
        };
    }
}

pub fn channel<T>() -> (Sender<T>, Receiver<T>) {
    let inner: Inner<T> = Inner::new();
    let inner = Arc::new(inner);
    return (
        Sender {
            inner: inner.clone(),
        },
        Receiver {
            inner: inner.clone(),
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
        assert_eq!(7, rx.recv());
    }
}
