use std::collections::VecDeque;
use std::io::ErrorKind;
use std::io::{BufRead, Write};
use std::net::SocketAddr;
use std::str::FromStr;

use anyhow::Result;
use bufstream::BufStream;
use std::net::TcpStream;

pub struct Connection {
    stream: BufStream<TcpStream>,
    tokens: VecDeque<String>,
    pub addr: SocketAddr,
}

impl Connection {
    pub fn new(tcp_stream: TcpStream, addr: SocketAddr) -> Self {
        let stream = BufStream::new(tcp_stream);
        Self {
            stream,
            tokens: Default::default(),
            addr,
        }
    }

    pub fn read_token(&mut self) -> Result<String> {
        loop {
            if let Some(token) = self.tokens.pop_front() {
                return Ok(token);
            }
            let mut line = String::new();
            let n = self.stream.read_line(&mut line)?;
            if n == 0 {
                return Err(anyhow::Error::msg("End of stream"));
            }
            self.tokens = line.trim().split(" ").map(|s| s.to_owned()).collect();
        }
    }

    pub fn read<T: FromStr>(&mut self) -> Result<T>
    where
        <T as FromStr>::Err: std::fmt::Debug,
    {
        let token = self.read_token()?;
        match token.parse::<T>() {
            Ok(res) => Ok(res),
            Err(err) => Err(std::io::Error::new(ErrorKind::InvalidData, format!("{err:?}")).into()),
        }
    }

    pub fn read_expect<T: ToString>(&mut self, expect: T) -> Result<()> {
        let token = self.read_token()?;
        let expect: String = expect.to_string();
        if token != expect {
            Err(anyhow::Error::msg(format!(
                "Expected to read {expect}, found {token}"
            )))
        } else {
            Ok(())
        }
    }

    #[must_use]
    pub fn write<T: std::fmt::Display>(&mut self, s: T) -> Result<()> {
        log::debug!("Sending to {}: {s}", self.addr);
        let s = format!("{}\n", s);
        let mut buf = s.as_bytes();
        while !buf.is_empty() {
            let n = self.stream.write(&buf)?;
            buf = &buf[n..];
        }
        self.stream.flush()?;
        return Ok(());
    }
}
