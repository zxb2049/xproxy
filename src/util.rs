use std::io::{self, ErrorKind};

use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{
        tcp::{ReadHalf, WriteHalf},
        TcpStream,
    },
};

use crate::DEFAULT_BUF_SIZE;

pub fn encrypt(bytes: &mut [u8], key: &[u8], offset: usize) {
    if key.len() == 0 {
        return;
    }

    for (idx, byte) in bytes.iter_mut().enumerate() {
        *byte ^= key[(idx + offset) % key.len()];
    }
}

pub async fn copy(r: &mut ReadHalf<'_>, w: &mut WriteHalf<'_>, key: &[u8]) -> io::Result<u64> {
    let mut buf = [0; DEFAULT_BUF_SIZE];
    let mut written = 0;
    let mut offset = 0;

    loop {
        let len = match r.read(&mut buf).await {
            Ok(0) => return Ok(written),
            Ok(len) => len,
            Err(ref e) if e.kind() == ErrorKind::Interrupted => continue,
            Err(e) => return Err(e),
        };
        encrypt(&mut buf[..len], key, offset);

        w.write_all(&buf[..len]).await?;
        written += len as u64;
        offset = (offset + len) % key.len();
    }
}

pub async fn flow(mut src: TcpStream, mut dst: TcpStream, key: &[u8]) -> io::Result<u64> {
    let (mut src_r, mut src_w) = src.split();
    let (mut dst_r, mut dst_w) = dst.split();

    let src_to_dst = async {
        let result = copy(&mut src_r, &mut dst_w, key).await;
        let _ = dst_w.shutdown().await;
        result
    };
    let dst_to_src = async {
        let result = copy(&mut dst_r, &mut src_w, key).await;
        let _ = src_w.shutdown().await;
        result
    };

    let result: io::Result<(u64, u64)> = tokio::try_join!(src_to_dst, dst_to_src);
    result.map(|(src_to_dst, dst_to_src)| src_to_dst.max(dst_to_src))
}
