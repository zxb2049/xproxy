use std::{
    io::{self, ErrorKind},
    sync::Arc,
    time::Duration,
};

use tokio::{io::AsyncWriteExt, net::TcpStream};

use crate::{
    prot::{XRequest, XResponse},
    util,
};

mod cfg;
pub use cfg::Config;

#[derive(Debug)]
pub struct Proxy {
    src: TcpStream,
    key: Arc<Vec<u8>>,
    hash: Arc<Vec<u8>>,
    time_out_sec: u64,
}

impl Proxy {
    pub fn new(src: TcpStream, key: Arc<Vec<u8>>, hash: Arc<Vec<u8>>, time_out_sec: u64) -> Self {
        Self {
            src,
            key,
            hash,
            time_out_sec,
        }
    }

    pub async fn tunnel(mut self) -> io::Result<()> {
        let dst = self.connect().await?;
        let flowed = util::flow(self.src, dst, &self.key).await?;

        #[cfg(debug_assertions)]
        println!("🟢Flowed {:?} Bytes", flowed);

        Ok(())
    }

    async fn connect(&mut self) -> io::Result<TcpStream> {
        let time_out = Duration::from_secs(self.time_out_sec);

        let xreq = match XRequest::read(&mut self.src, &self.hash, time_out).await {
            Ok(xreq) => xreq,
            Err(e) if e.kind() == ErrorKind::TimedOut => return Err(e),
            Err(e) => {
                self.src.write_all(&XResponse::BadHash.to_vec()).await?;
                return Err(e);
            }
        };

        let host = xreq.to_host(&self.key, &self.hash);

        match TcpStream::connect(&host).await {
            Ok(dst) => {
                self.src.write_all(&XResponse::Ok.to_vec()).await?;
                Ok(dst)
            }
            Err(e) => {
                self.src.write_all(&XResponse::NotFound.to_vec()).await?;
                Err(e)
            }
        }
    }
}
