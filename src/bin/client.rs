use std::io;

use tokio::net::TcpListener;
use xproxy::local::{Config, Proxy};

#[tokio::main]
async fn main() -> io::Result<()> {
    let config = Config::read().await?;

    let listener = TcpListener::bind(config.addr()).await?;
    loop {
        let (src, _) = listener.accept().await?;

        let proxy = Proxy::new(src, config.key(), config.hash());
        let server_addr = config.server_addr();

        tokio::spawn(async move {
            if let Err(e) = proxy.tunnel(&server_addr).await {
                println!("{:?}", e);
            }
        });
    }
}
