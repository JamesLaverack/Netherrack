//! Functions relating to network operations.
//!
//! This will eventually be moved

pub mod deque_buffer;

use std::collections::VecDeque;

use std::net::{Shutdown, TcpListener, TcpStream};

use std::thread;

use std::io::Read;

use self::deque_buffer::DequeBuffer;

fn start_listening(port: u16) -> Result<TcpListener, &'static str> {

    let listener = TcpListener::bind(("127.0.0.1", port));
    
    if listener.is_ok() {
        return Ok(listener.unwrap());
    } else {
        return Err("Could not listen on requested port");
    }

}

pub fn start_network() {

    let listener = start_listening(25565);
    
    if listener.is_err() {
        error!("Could not start listening on port {}", 25565);
    }
    
    let listener = listener.unwrap();
    
    for stream in listener.incoming() {
    
        match stream {
            Ok(stream) => {
                thread::spawn(move || {
                    // New connection succeeded
                    handle_new_client(stream)
                });
            }
            Err(e) => { 
                error!("Could not accept a new stream: {}", e);
            }
        }
    
    }
    
    drop(listener);

}

pub fn handle_new_client(mut stream: TcpStream) {
    info!("Got a new client from {}! Oh my!", stream.peer_addr().unwrap());
    
    loop {
    
        let mut raw_buffer = vec![0u8; 512];
        
        match stream.read(&mut raw_buffer) {
        
            Err(error) => {
                // Connection dropped out or something broke
                // In the future, we use the customized NetherrackStream to see if we're in GAME mode, and if so, get the attached player and start saving their information and get rid of them
                // For now, we just exit the loop
                error!("Got an error! {}", error);
                
                let _ = stream.shutdown(Shutdown::Both);
                break; //Close the connection
            }
            Ok(count) => {
                if count == 0 {
                    //Nothing in stream, let's sleep for a few milliseconds and try again
                    ::std::thread::sleep_ms(2);
                } else {
                
                    // Now truncate the raw buffer - Don't worry, it will reallocate to 512 on the next networking loop
                    raw_buffer.truncate(count);
                    
                    let mut backing: VecDeque<u8> = VecDeque::<u8>::with_capacity(count);
                    
                    for b in raw_buffer {
                        
                        backing.push_back(b);
                    }
                
                    let mut buffer = DequeBuffer::from(backing);
                    
                    //drop(raw_buffer);
                
                    info!("Read {} bytes, information is {}: {}: {}: {}", count, buffer.read_signed_byte(), buffer.read_signed_byte(), buffer.read_signed_byte(), buffer.read_signed_byte());
                    
                    info!("Real buffer size is {}", buffer.remaining());
                    
                    // Okay, to prevent a memory leak, we'll destroy the connection... For now.
                    // In the future, we'll need to reference the ping/pong packets and kick it off after, say, 20 seconds
                    info!("Shutdown result: {:?}", stream.shutdown(Shutdown::Both));
                    break;
                }
            }
        
        }
    
    }
    
    info!("Connection closed");
    
}