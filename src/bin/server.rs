use std::io;

use tokio::net::TcpListener;
use xproxy::remote::{Config, Proxy};

#[tokio::main]
async fn main() -> io::Result<()> {
    let config = Config::read().await?;

    let listener = TcpListener::bind(config.addr()).await?;
    loop {
        let (src, _) = listener.accept().await?;

        let proxy = Proxy::new(src, config.key(), config.hash(), config.time_out_sec());

        tokio::spawn(async move {
            #[cfg(not(debug_assertions))]
            proxy.tunnel().await;

            #[cfg(debug_assertions)]
            if let Err(e) = proxy.tunnel().await {
                println!("{:?}", e);
            }
        });
    }
}
