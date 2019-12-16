use std::fs::OpenOptions;
use std::io;
use std::io::Read;
use std::io::Write;
use std::net::{TcpListener, TcpStream};
use std::str;
use std::sync::mpsc::channel;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::Sender;
use std::thread;
use threadpool::ThreadPool;

fn handle_client(mut stream: TcpStream, sender: &Sender<String>) {
    loop {
        let mut buf = [0; 1024];
        match stream.read(&mut buf) {
            Ok(n) => {
                if n == 0 {
                    println!("close connection");
                    break;
                }

                let s = str::from_utf8(&buf[0..n]).unwrap();
                sender.send(String::from(s)).unwrap();

                match stream.write_all(&buf[0..n]) {
                    Ok(_) => {}
                    Err(e) => panic!("{}", e),
                }
            }
            Err(e) => panic!("{}", e),
        }
    }
}

fn main() -> io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:8081")?;

    let n_workers = 2;
    let pool = ThreadPool::new(n_workers);
    let (sender, receiver): (Sender<String>, Receiver<String>) = channel();

    thread::spawn(move || {
        for stream in listener.incoming() {
            match stream {
                Ok(s) => {
                    let sender = sender.clone();
                    pool.execute(move || {
                        handle_client(s, &sender);
                    });
                }
                Err(e) => println!("error: {}", e),
            }
        }
    });

    let mut f = OpenOptions::new()
        .write(true)
        .create(true)
        .open("/tmp/shiodome-rs.txt")
        .unwrap();

    for l in receiver.iter() {
        println!("{}", l);
        f.write(l.as_bytes()).unwrap();
    }

    Ok(())
}
