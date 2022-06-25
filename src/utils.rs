use directories::BaseDirs;
use rosc::{OscType, encoder, OscPacket, OscMessage};
use std::{path::Path, net::UdpSocket};

// THANKS SUTEKH
pub fn dir_exists(p: &String) -> bool {
    Path::new(&p).is_dir()
}
pub fn file_exists(p: &String) -> bool {
    Path::new(&p).is_file()
}
pub fn get_user_home_dir() -> String {
    let bd = BaseDirs::new().expect("[-] Could not get user's directories.");
    let bd = bd.home_dir().to_str().expect("[-] Failed to get user's home directory.");
    bd.to_string()
}

// Send OSC data
pub fn send_data(sock: &UdpSocket, param_name: &str, param_arg: OscType) {
    let full_param_name = format!("/avatar/parameters/{}", &param_name);

    if !&param_name.contains("OSC/Enabled") {
        println!("  ==> Sending '{}' as {:?}", &full_param_name, param_arg);
    }

    // Create OSC/1.0 Message buffer with parameter name and parameter value/arg
    let msg_buf = encoder::encode(&OscPacket::Message(OscMessage {
        addr: full_param_name,
        args: vec![param_arg],
    }))
    .unwrap();

    // Sends the encoded Message buffer to VRChat on port 9000
    sock.send_to(&msg_buf, "127.0.0.1:9000").unwrap();
}
