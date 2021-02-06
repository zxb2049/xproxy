use std::{io, net::SocketAddr, sync::Arc};

use prot::Protocol;
use tokio::{io::AsyncWriteExt, net::TcpStream};

use crate::{
    prot::{XRequest, XResponse},
    util,
};

mod cfg;
pub use cfg::Config;
mod prot;

#[derive(Debug)]
pub struct Proxy {
    src: TcpStream,
    key: Arc<Vec<u8>>,
    hash: Arc<String>,
}

impl Proxy {
    pub fn new(src: TcpStream, key: Arc<Vec<u8>>, hash: Arc<String>) -> Self {
        Self { src, key, hash }
    }

    pub async fn tunnel(mut self, server_addr: &SocketAddr) -> io::Result<()> {
        let dst = self.connect(server_addr).await?;
        let flowed = util::flow(self.src, dst, &self.key).await?;

        println!("🟢Flowed {:?} Bytes", flowed);
        Ok(())
    }

    async fn connect(&mut self, server_addr: &SocketAddr) -> io::Result<TcpStream> {
        let host = Protocol::check(&self.src)
            .await?
            .to_host(&mut self.src)
            .await?;

        let mut dst = TcpStream::connect(server_addr).await?;

        let xreq = XRequest::new(host, &self.key, &self.hash);
        dst.write_all(&xreq.to_vec()).await?;
        XResponse::is_ok(&mut self.src, &mut dst).await?;

        Ok(dst)
    }
}
