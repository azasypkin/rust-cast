pub mod connection;
pub mod heartbeat;
pub mod media;
pub mod receiver;

#[cfg(test)]
mod tests {
    use std::io::{Read, Write};

    pub struct MockTcpStream {
        pub read_buffer: Vec<u8>,
        pub read_pos: usize,
        pub write_buffer: Vec<u8>,
    }

    impl MockTcpStream {
        pub fn new() -> Self {
            MockTcpStream {
                read_buffer: vec![],
                read_pos: 0,
                write_buffer: vec![],
            }
        }
    }

    impl Read for MockTcpStream {
        fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
            let bytes_to_read = std::cmp::min(buf.len(), self.read_buffer.len() - self.read_pos);
            buf[..bytes_to_read]
                .copy_from_slice(&self.read_buffer[self.read_pos..self.read_pos + bytes_to_read]);
            self.read_pos += bytes_to_read;
            Ok(bytes_to_read)
        }
    }

    impl Write for MockTcpStream {
        fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
            self.write_buffer.extend_from_slice(buf);
            Ok(buf.len())
        }

        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }
}
