#![allow(dead_code, unused_variables)]

use std::future::Future;

fn main() {
    // one thead for one waiting operation
    let read_from_terminal = std::thread::spawn(move || {
        let mut x = std::io::Stdin::lock(std::io::stdin());
        for line in x.lines() {
            // do something
        }
    });

    // one thead for one waiting operation
    let read_from_network = std::thread::spawn(move || {
        let mut x = std::net::TcpListener::bind("0.0.0.0:8080").unwrap();
        while let Ok(stream) = x.accept() {
            // do something
        }
    });

    // Can be way more threads which is complicated and overwhelms the system
    let network = read_from_network();
    let terminal = read_from_terminal();

    select ! {
        stream <- network.await => {
            // do something with network stream
        }

        line <- terminal.await => {
            // do something with line
        }
    }


}

as

async fn foo1() -> usize {
    0
}

// Future trait signifies a value that will eveutally be a usize
// Similar to a proice in jS

async fn foo() -> impl Future<Output = usize> {
    async {
        // First run
        println!("foo");
        foo1().await; // wait here until future gets resolved
                      // Second run
        println!("foo foo");
        foo1().await; // wait here until future gets resolved

        0
    }
}
