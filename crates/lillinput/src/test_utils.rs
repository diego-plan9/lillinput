#[cfg(test)]
use std::env;
use std::io::prelude::*;
use std::os::unix::net::{UnixListener, UnixStream};
use std::sync::{Arc, Mutex};
use std::thread;

use tempfile::{Builder, NamedTempFile};

static MSG_COMMAND: u32 = 0;
static MSG_VERSION: u32 = 7;

/// Container for I3 IPC messages.
struct I3IpcMessage {
    message_type: u32,
    message_payload: String,
}

/// Create a new message to be sent to `i3`.
///
/// # Arguments
///
/// * `payload` - content of the i3 message.
/// * `message_type` - type of i3 message.
fn create_i3_message(payload: &[u8], message_type: u32) -> Vec<u8> {
    // From https://i3wm.org/docs/ipc.html#_sending_messages_to_i3.
    let magic_string: &[u8] = b"i3-ipc";
    #[allow(clippy::cast_possible_truncation)]
    let length: &[u8] = &(payload.len() as u32).to_ne_bytes();
    let message_type: &[u8] = &message_type.to_ne_bytes();

    [magic_string, length, message_type, payload].concat()
}

/// Parse a message received from i3 into a [`I3IpcMessage`].
///
/// # Arguments
///
/// * `socket` - [`UnixStream`] with the message.
fn parse_i3_message(mut socket: &UnixStream) -> Option<I3IpcMessage> {
    // Read the message from the client.
    let mut buffer = [0u8; 14];
    let mut dest_size = [0u8; 4];
    let mut dest_type = [0u8; 4];

    // Retrieve the message size and type.
    let bytes_read = socket.read(&mut buffer).ok();
    if bytes_read == Some(0) {
        // TODO: allow read() to block until there is content.
        return None;
    }

    dest_size.clone_from_slice(&buffer[6..10]);
    dest_type.clone_from_slice(&buffer[10..14]);
    let message_size = u32::from_ne_bytes(dest_size);
    let message_type = u32::from_ne_bytes(dest_type);

    // Consume the payload.
    let mut dest_payload = vec![0u8; message_size as usize];
    if message_size > 0 {
        socket.read(&mut dest_payload).ok();
    }

    let payload_string = String::from_utf8_lossy(&dest_payload).into_owned();

    // Return the parsed message.
    Some(I3IpcMessage {
        message_type,
        message_payload: payload_string,
    })
}

/// Create a fake reply to be sent to i3.
///
/// If the message is not among the expected ones, this method will
/// return None.
///
/// # Arguments
///
/// * `message_type` - type of message.
fn create_i3_reply(message_type: u32) -> Option<Vec<u8>> {
    if message_type == MSG_VERSION {
        Some(create_i3_message(
            r#"{
               "human_readable" : "4.2-fake",
               "loaded_config_file_name" : "/tmp/fake/.i3.i3/config",
               "minor" : 2,
               "patch" : 0,
               "major" : 4
            }"#
            .as_bytes(),
            MSG_VERSION,
        ))
    } else if message_type == MSG_COMMAND {
        Some(create_i3_message(
            r#"[{ "success": true }]"#.as_bytes(),
            MSG_COMMAND,
        ))
    } else {
        None
    }
}

/// Initialize the RPC listener.
///
/// Start a RPC listener which receives the i3 messages, mimicking a
/// small subset of the i3 RPC protocol. The listener parses the messages
/// received, storing them in a shared variable, and creates fake replies
/// accordingly.
///
/// The `I3SOCK` environment variable is set during this function, which
/// is used by `I3Connection` to determine the i3 socket.
///
/// # Arguments
///
/// * `message_log` - type of message.
///
/// # Panics
///
/// This function will panic if the creation of the temporary socket files or
/// listener is not successful.
///
/// # Returns
///
/// The file with the temporary i3 socket.
#[must_use]
pub fn init_listener(message_log: Arc<Mutex<Vec<String>>>) -> NamedTempFile {
    let socket_file = Builder::new().tempfile().unwrap();
    let socket_file_name = socket_file.path().file_name().unwrap();
    // Use a custom listener instead of the i3 socket.
    let listener = UnixListener::bind(socket_file_name).unwrap();
    // Trick I3Connection::connect() into using the custom listener.
    env::set_var("I3SOCK", socket_file_name);

    thread::spawn(move || {
        match listener.accept() {
            Ok((mut socket, _)) => {
                loop {
                    if let Some(i3_message) = parse_i3_message(&socket) {
                        if let Some(reply) = create_i3_reply(i3_message.message_type) {
                            // Add the message to the log.
                            let mut messages = message_log.lock().unwrap();
                            messages.push(i3_message.message_payload);
                            std::mem::drop(messages);

                            // Send the reply.
                            socket.write_all(&reply).ok();
                        }
                    }
                }
            }
            Err(e) => println!("accept function failed: {:?}", e),
        };
    });

    socket_file
}
