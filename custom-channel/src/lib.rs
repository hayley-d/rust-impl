use std::collections::VecDeque;
use std::sync::{Arc, Condvar, Mutex};

pub struct SyncSender<T> {
    shared: Arc<Shared<T>>,
}

impl<T> SyncSender<T> {
    pub fn send(&mut self, t: T) {
        let mut inner = self.shared.inner.lock().unwrap();
        inner.queue.push_back(t);
        drop(inner);
        self.shared.not_empty.notify_one();
    }
}

impl<T> Clone for SyncSender<T> {
    fn clone(&self) -> Self {
        let mut inner = self.shared.inner.lock().unwrap();
        inner.senders += 1;
        drop(inner);
        return SyncSender {
            shared: Arc::clone(&self.shared),
        };
    }
}
pub struct Sender<T> {
    shared: Arc<Shared<T>>,
}

impl<T> Sender<T> {
    pub fn send(&mut self, t: T) {
        let mut inner = self.shared.inner.lock().unwrap();
        if inner.queue.len() > inner.bound {
            inner = self.shared.not_full.wait(inner).unwrap();
        }
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
    buffer: VecDeque<T>,
}

impl<T> Receiver<T> {
    pub fn recv(&mut self) -> Option<T> {
        if let Some(t) = self.buffer.pop_front() {
            return Some(t);
        }

        let mut inner = self.shared.inner.lock().unwrap();
        loop {
            match inner.queue.pop_front() {
                Some(t) => {
                    if !inner.queue.is_empty() {
                        std::mem::swap(&mut self.buffer, &mut inner.queue);
                    }
                    self.shared.not_full.notify_all();
                    return Some(t);
                }
                None if Arc::strong_count(&self.shared) == 1 => return None,
                None => {
                    // wait on the condition
                    // not a spin lock
                    inner = self.shared.not_empty.wait(inner).unwrap();
                }
            }
        }
    }
}

impl<T> Iterator for Receiver<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        return self.recv();
    }
}

pub struct Inner<T> {
    pub queue: VecDeque<T>,
    pub senders: usize,
    pub bound: usize,
}

pub struct Shared<T> {
    pub inner: Mutex<Inner<T>>,
    pub not_empty: Condvar,
    pub not_full: Condvar,
}

impl<T> Shared<T> {
    pub fn new() -> Shared<T> {
        return Shared {
            inner: Mutex::new(Inner::new()),
            not_empty: Condvar::new(),
            not_full: Condvar::new(),
        };
    }

    pub fn new_bounded(bound: usize) -> Shared<T> {
        return Shared {
            inner: Mutex::new(Inner::new_bounded(bound)),
            not_empty: Condvar::new(),
            not_full: Condvar::new(),
        };
    }
}

impl<T> Inner<T> {
    pub fn new() -> Inner<T> {
        return Inner {
            queue: VecDeque::new(),
            senders: 1,
            bound: 0,
        };
    }

    pub fn new_bounded(bound: usize) -> Inner<T> {
        return Inner {
            queue: VecDeque::new(),
            senders: 1,
            bound,
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
            buffer: VecDeque::new(),
        },
    );
}

pub fn sync_channel<T>(bound: usize) -> (SyncSender<T>, Receiver<T>) {
    let shared: Shared<T> = Shared::new();
    let shared = Arc::new(shared);

    return (
        SyncSender {
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
