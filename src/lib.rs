use serde::{Deserialize, Serialize};
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
    pub fn connect<P: AsRef<Path>>(path: P, timeout: Option<Duration>) -> Result<I3Stream, Error> {
        match UnixStream::connect(path) {
            Ok(socket) => {
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

pub enum FocusTarget {
    Previous,
    Next,
}

pub struct I3Service {
    i3stream: I3Stream,
}

#[derive(Serialize, Deserialize)]
struct I3TreeNode {
    id: u64,
    focused: bool,
    nodes: Vec<I3TreeNode>,
}

impl I3Service {
    pub fn connect<P: AsRef<Path>>(path: P, timeout: Option<Duration>) -> Result<I3Service, Error> {
        I3Stream::connect(path, timeout).map(|i3stream| I3Service { i3stream })
    }

    pub fn focus(&mut self, target: FocusTarget) -> Result<(), Error> {
        let payload = self.i3stream.execute(I3Message::GetTree)?;
        let node: I3TreeNode = serde_json::from_slice(&payload)?;

        match I3Service::find_node_to_focus(&node, &target) {
            Some(node) => {
                let payload = format!("[con_id={}]focus", node.id).into_bytes();
                match self.i3stream.execute(I3Message::RunCommand(payload)) {
                    Ok(_) => Ok(()),
                    Err(err) => Err(err),
                }
            }
            None => Ok(()),
        }
    }

    fn find_node_to_focus<'a>(
        node: &'a I3TreeNode,
        target: &FocusTarget,
    ) -> Option<&'a I3TreeNode> {
        for (i, child) in node.nodes.iter().enumerate() {
            if child.focused {
                let index = match target {
                    FocusTarget::Previous => (i + node.nodes.len() - 1) % node.nodes.len(),
                    FocusTarget::Next => (i + 1) % node.nodes.len(),
                };

                return Some(&node.nodes[index]);
            } else {
                match I3Service::find_node_to_focus(child, target) {
                    Some(node) => return Some(node),
                    None => continue,
                }
            }
        }

        None
    }
}
