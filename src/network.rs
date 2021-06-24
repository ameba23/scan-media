use async_std::{
    prelude::*,
    task,
    net::{TcpListener, ToSocketAddrs, TcpStream},
};
use async_std_utp::{UtpListener, UtpSocket};

use futures_lite::io::{AsyncRead, AsyncWrite};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

async fn accept_loop(addr: impl ToSocketAddrs) -> Result<()> {
    let listener = UtpListener::bind(addr).await?;
    let mut incoming = listener.incoming();
    println!("Listening {:?}", listener.local_addr().unwrap());
    while let Some(connection) = incoming.next().await {
        match connection {
            Ok((socket, src)) => {
                // let stream = stream?;
                println!("Connection from {}", src);
                task::spawn(handle_client(socket));
                // let _handle = task::spawn(connection_loop(stream));
            }
            _ => {},
        }
    }
    Ok(())
}
async fn handle_client(mut s: UtpSocket) {
    let mut buf = vec![0; 1500];

    // Reply to a data packet with its own payload, then end the connection
    match s.recv_from(&mut buf).await {
        Ok((nread, src)) => {
            println!("<= [{}] {:?}", src, &buf[..nread]);
            drop(s.send_to(&buf[..nread]).await);
        }
        Err(e) => println!("{}", e),
    }
}
// async fn connection_loop <IO> (stream: IO) -> Result<()>
// where
//     IO: AsyncWrite + AsyncRead + Send + Unpin + 'static,
// {
//   Ok(())
// }

pub fn run() -> Result<()> {
    task::block_on(accept_loop("0.0.0.0:29394"))
}
