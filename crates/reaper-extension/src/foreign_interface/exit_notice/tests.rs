use std::io::Read;
use std::net::{TcpListener, TcpStream};

use protocol::message::{ServerMessage, decode_message};

use super::*;

type TestResult = Result<(), Box<dyn std::error::Error>>;

#[cfg(unix)]
fn handle_of(stream: &TcpStream) -> usize {
    use std::os::fd::AsRawFd;
    usize::try_from(stream.as_raw_fd()).unwrap_or(0)
}

#[cfg(windows)]
fn handle_of(stream: &TcpStream) -> usize {
    use std::os::windows::io::AsRawSocket;
    usize::try_from(stream.as_raw_socket()).unwrap_or(0)
}

#[test]
fn the_frame_reaches_a_client_that_reads_it_like_any_other_message() -> TestResult {
    let listener = TcpListener::bind(("127.0.0.1", 0))?;
    let mut client = TcpStream::connect(listener.local_addr()?)?;
    let (server_side, _) = listener.accept()?;

    let frame = link::closing_frame();
    write_frame(handle_of(&server_side), &frame);

    let mut received = vec![0_u8; frame.len()];
    client.read_exact(&mut received)?;
    assert_eq!(received, frame);
    let payload = received.get(4..).ok_or("frame without a payload")?;
    assert_eq!(
        decode_message::<ServerMessage>(payload)?,
        ServerMessage::Closing
    );
    Ok(())
}

#[test]
fn a_peer_that_is_gone_does_not_hurt() -> TestResult {
    let listener = TcpListener::bind(("127.0.0.1", 0))?;
    let client = TcpStream::connect(listener.local_addr()?)?;
    let (server_side, _) = listener.accept()?;
    drop(client);
    let frame = link::closing_frame();
    write_frame(handle_of(&server_side), &frame);
    write_frame(handle_of(&server_side), &frame);
    Ok(())
}
