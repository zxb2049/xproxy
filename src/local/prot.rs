use std::io::{self, Error, ErrorKind};

use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};

use crate::DEFAULT_BUF_SIZE;

pub enum Protocol {
    Http,
    Https,
}

impl Protocol {
    pub async fn check(src: &TcpStream) -> io::Result<Self> {
        let mut buf = [0; DEFAULT_BUF_SIZE];
        let len = src.peek(&mut buf).await?;
        let msg = String::from_utf8_lossy(&buf[..len]);

        if is_https(&msg) {
            Ok(Protocol::Https)
        } else if is_http(&msg) {
            Ok(Protocol::Http)
        } else {
            Err(Error::new(
                ErrorKind::Other,
                "🔴暂只支持http代理、tunnel代理",
            ))
        }
    }

    pub async fn to_host(self, src: &mut TcpStream) -> io::Result<String> {
        let mut buf = [0; DEFAULT_BUF_SIZE];
        let len = match self {
            Self::Http => src.peek(&mut buf).await?,
            Self::Https => src.read(&mut buf).await?,
        };
        let msg = String::from_utf8_lossy(&buf[..len]);

        let host = host(&msg).ok_or(Error::new(ErrorKind::Other, "🔴http请求报文找不到host"))?;

        match self {
            Self::Http => Ok(format!("{}:80", host)),
            Self::Https => {
                src.write_all(b"HTTP/1.1 200 OK\r\n\r\n").await?;
                Ok(host.to_string())
            }
        }
    }
}

fn is_http(msg: &str) -> bool {
    msg.contains("\r\n\r\n") && msg.contains("\r\nHost: ")
}

fn is_https(msg: &str) -> bool {
    is_http(msg) && msg.starts_with("CONNECT")
}

fn host(msg: &str) -> Option<&str> {
    msg.find("\r\n\r\n")
        .map(|end| &msg[..end])
        .map(|headers| headers.lines())
        .and_then(|mut lines| lines.find(|l| l.starts_with("Host")))
        .and_then(|host| host.strip_prefix("Host: "))
}
