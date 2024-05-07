pub mod connection;
pub mod heartbeat;
pub mod media;
pub mod receiver;

#[cfg(test)]
mod tests {
    use byteorder::{BigEndian, WriteBytesExt};
    use std::io::{Read, Write};

    pub struct MockTcpStream {
        pub read_buffer_len: Option<Vec<u8>>,
        pub read_buffer: Vec<u8>,
        pub write_buffer: Vec<u8>,
    }

    impl MockTcpStream {
        pub fn new() -> Self {
            MockTcpStream {
                read_buffer_len: None,
                read_buffer: vec![],
                write_buffer: vec![],
            }
        }

        pub fn set_message<M: protobuf::Message>(&mut self, message: M) {
            let message = message.write_to_bytes().unwrap();
            let mut len = Vec::<u8>::new();
            len.write_u32::<BigEndian>(message.len() as u32).unwrap();

            self.read_buffer_len = Some(len);
            self.read_buffer = message;
        }
    }

    impl Read for MockTcpStream {
        fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
            return if let Some(len) = self.read_buffer_len.take() {
                buf[..4].copy_from_slice(len.as_slice());
                Ok(4)
            } else {
                let len = self.read_buffer.len();
                buf[..len].copy_from_slice(self.read_buffer.as_slice());
                Ok(len)
            };
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
