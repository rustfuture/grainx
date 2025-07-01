use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use std::error::Error;

pub async fn start_server(addr: &str) -> Result<(), Box<dyn Error>> {
    let listener = TcpListener::bind(addr).await?;
    println!("Server listening on {}", addr);

    loop {
        let (mut socket, addr) = listener.accept().await?;
        println!("New client connected from {}", addr);

        tokio::spawn(async move {
            let mut buf = vec![0; 1024];
            loop {
                let n = match socket.read(&mut buf).await {
                    Ok(n) if n == 0 => return,
                    Ok(n) => n,
                    Err(e) => {
                        eprintln!("failed to read from socket; err = {:?}", e);
                        return;
                    }
                };

                let msg = String::from_utf8_lossy(&buf[..n]);
                println!("Received from {}: {}", addr, msg);

                if let Err(e) = socket.write_all(b"ACK").await {
                    eprintln!("failed to write to socket; err = {:?}", e);
                    return;
                }
            }
        });
    }
}

pub async fn start_agent(addr: &str, message: &str) -> Result<(), Box<dyn Error>> {
    let mut stream = TcpStream::connect(addr).await?;
    println!("Agent connected to {}", addr);

    if let Err(e) = stream.write_all(message.as_bytes()).await {
        eprintln!("failed to write to socket; err = {:?}", e);
        return Err(e.into());
    }

    let mut buf = vec![0; 1024];
    let n = stream.read(&mut buf).await?;
    let response = String::from_utf8_lossy(&buf[..n]);
    println!("Agent received: {}", response);

    Ok(())
}
