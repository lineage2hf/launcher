use std::env;
use std::io;
use std::net::{TcpListener, TcpStream, ToSocketAddrs};
use std::path::PathBuf;
use std::process::Command;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

const LOGIN_REMOTE: &str = "trolley.proxy.rlwy.net:46458";
const GAME_REMOTE: &str = "ballast.proxy.rlwy.net:32374";
const LOGIN_LOCAL: &str = "127.0.0.1:2106";
const GAME_LOCAL: &str = "127.0.0.1:7777";

fn pipe(mut src: TcpStream, mut dst: TcpStream) {
    if let Err(e) = io::copy(&mut src, &mut dst) {
        eprintln!("[ERROR] pipe: {e}");
    }
    let _ = dst.shutdown(std::net::Shutdown::Write);
}

fn handle_conn(client: TcpStream, remote: &str) {
    let server = match TcpStream::connect(remote) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("[ERROR] Cannot connect to {remote}: {e}");
            return;
        }
    };

    let _ = client.set_nodelay(true);
    let _ = server.set_nodelay(true);

    let client_clone = client.try_clone().expect("clone client");
    let server_clone = server.try_clone().expect("clone server");

    thread::spawn(move || pipe(client, server));
    thread::spawn(move || pipe(server_clone, client_clone));
}

fn proxy(local: &str, remote: &'static str, shutdown: Arc<AtomicBool>) {
    let listener = match TcpListener::bind(local) {
        Ok(l) => l,
        Err(e) => {
            eprintln!("[ERROR] Cannot listen on {local}: {e}");
            return;
        }
    };
    listener
        .set_nonblocking(true)
        .expect("set_nonblocking failed");

    println!("[OK] {local} -> {remote}");

    while !shutdown.load(Ordering::Relaxed) {
        match listener.accept() {
            Ok((stream, _)) => {
                let _ = stream.set_nonblocking(false);
                let remote = remote.to_string();
                thread::spawn(move || handle_conn(stream, &remote));
            }
            Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                thread::sleep(Duration::from_millis(50));
            }
            Err(_) => continue,
        }
    }
}

fn find_l2bin() -> Option<PathBuf> {
    let cwd = env::current_dir().ok()?;
    let path = cwd.join("L2.bin");
    if path.exists() {
        return Some(path);
    }

    let exe = env::current_exe().ok()?;
    let dir = exe.parent()?;
    let path = dir.join("L2.bin");
    if path.exists() {
        return Some(path);
    }

    None
}

fn wait_for_enter() {
    println!("\nPress Enter to exit...");
    let mut buf = String::new();
    let _ = io::stdin().read_line(&mut buf);
}

fn main() {
    let _ = LOGIN_REMOTE.to_socket_addrs();
    let _ = GAME_REMOTE.to_socket_addrs();

    let shutdown = Arc::new(AtomicBool::new(false));

    let s1 = Arc::clone(&shutdown);
    let t1 = thread::spawn(move || proxy(LOGIN_LOCAL, LOGIN_REMOTE, s1));

    let s2 = Arc::clone(&shutdown);
    let t2 = thread::spawn(move || proxy(GAME_LOCAL, GAME_REMOTE, s2));

    thread::sleep(Duration::from_millis(500));

    match find_l2bin() {
        None => {
            println!("[WARN] L2.bin not found!");
            println!("[INFO] Put this launcher in the same folder as L2.bin");
            println!("[INFO] Port forwarding is active. Press Enter to stop.");
            println!();
            wait_for_enter();
        }
        Some(l2bin) => {
            println!("[OK] Found: {}", l2bin.display());
            println!("[OK] Starting Lineage 2...");
            println!();

            let dir = l2bin.parent().unwrap();
            match Command::new(&l2bin)
                .arg("IP=127.0.0.1")
                .current_dir(dir)
                .spawn()
            {
                Err(e) => {
                    eprintln!("[ERROR] Cannot start L2.bin: {e}");
                    wait_for_enter();
                }
                Ok(mut child) => {
                    println!("[INFO] Game started. This window will stay open for connection forwarding.");
                    println!("[INFO] Do NOT close this window while playing!");
                    println!();
                    let _ = child.wait();
                }
            }
        }
    }

    println!("\n[INFO] Shutting down...");
    shutdown.store(true, Ordering::Relaxed);
    let _ = t1.join();
    let _ = t2.join();
}
