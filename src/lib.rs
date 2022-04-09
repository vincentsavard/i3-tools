use std::io::{Error, Read, Write};
use std::os::unix::net::UnixStream;
use std::path::Path;
use std::time::Duration;

pub enum I3Message {
    RunCommand(Vec<u8>),
    GetTree,
}

pub struct I3Stream {
    socket: UnixStream,
}

impl I3Stream {
    pub fn connect<P: AsRef<Path>>(path: P) -> Result<I3Stream, Error> {
        match UnixStream::connect(path) {
            Ok(socket) => {
                let timeout = Some(Duration::from_millis(250));
                socket.set_read_timeout(timeout).unwrap();
                socket.set_write_timeout(timeout).unwrap();

                Ok(I3Stream { socket })
            }
            Err(error) => Err(error),
        }
    }

    pub fn execute(&mut self, message: I3Message) -> Result<Vec<u8>, Error> {
        self.write(message)?;
        self.read()
    }

    fn write(&mut self, message: I3Message) -> Result<(), Error> {
        let (payload_length, message_type, payload) = match message {
            I3Message::RunCommand(payload) => (
                (payload.len() as u32).to_ne_bytes(),
                0_u32.to_ne_bytes(),
                payload,
            ),
            I3Message::GetTree => (0_u32.to_ne_bytes(), 4_u32.to_ne_bytes(), vec![]),
        };

        self.socket.write_all(b"i3-ipc")?;
        self.socket.write_all(&payload_length)?;
        self.socket.write_all(&message_type)?;
        self.socket.write_all(&payload)?;

        Ok(())
    }

    fn read(&mut self) -> Result<Vec<u8>, Error> {
        let mut response_payload_length = [0_u8; 4];

        self.socket.read_exact(&mut [0_u8; 6])?; // Skip magic bytes
        self.socket.read_exact(&mut response_payload_length)?;
        self.socket.read_exact(&mut [0_u8; 4])?; // Skip message type

        let response_payload_length = u32::from_ne_bytes(response_payload_length);
        let mut response_payload = vec![0_u8; response_payload_length as usize];
        self.socket.read_exact(&mut response_payload)?;

        Ok(response_payload)
    }
}
