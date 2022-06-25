// Rust OSC library import
extern crate rosc;

use ass::{AssParam, send_state};
// Serde
use serde::{Serialize, Deserialize};
//use serde_json::{Serializer, Deserializer, to_string, Value};
use serde_json::{self, Value};
// ROSC encoder import
use rosc::encoder;
// ROSC types (Message types / Packet types / ROSC arg types
use rosc::{OscMessage, OscPacket, OscType};
use std::collections::HashMap;
use std::fs::read_dir;
// IPv4 Address / Udp Socket object
use std::net::{SocketAddrV4, UdpSocket};
// From String Trait
use std::str::FromStr;
// Duration struct
use std::time::Duration;
// Proc Environment / 32 bit float / thread lib
use std::io::{Write, Read};
use std::{fs, env, f32, thread};
use std::path::Path;
use directories::BaseDirs;

// OpenVR
#[path = "ovr.rs"] mod ovr;
use openvr::{TrackedControllerRole};

// ASS
#[path = "ass.rs"] mod ass;

// Utils
#[path = "utils.rs"] mod util;

/*
    Pulls the avatar ID from an OSC message
*/
fn parse_id(args: Vec<OscType>) -> Option<String> {
    match &args[0] {
        OscType::String(av_id) => {
            return Some(String::from(av_id));
        },
        _ => {
            return None;
        }
    }
}

/*
    Takes in serde_json value and converts it to its
    cooresponding OscType value
 */
fn val_to_osc(v: &Value) -> OscType {
    match v {
        serde_json::Value::Bool(b) => return OscType::Bool(*b),
        serde_json::Value::Number(n) => return num_to_osc(n),
        _ => return OscType::Nil
    };

    fn num_to_osc(n: &serde_json::Number) -> OscType {
        if n.is_f64() {
            match n.as_f64() {
                Some(f) => return OscType::Float(f as f32),
                _ => OscType::Nil
            }
        } else if n.is_i64() {
            match n.as_i64() {
                Some(i) => return OscType::Int(i as i32),
                _ => OscType::Nil
            }
        } else {
            return OscType::Nil;
        }
    }
}


/*
    Main program loop
 */
fn start(sock: &UdpSocket, ass_dir: String, remi_ovr: ovr::RemiOVR) {

    // Create/allocate buffer on the stack with a size of MTU
    let mut buf = [0u8; rosc::decoder::MTU];
    
    // Current Avatar State
    let mut avatar_id = String::new();
    let mut state: HashMap<String, ass::AssParam> = HashMap::new();
    let mut is_saving: bool = false;

    // Continuously read OSC data from port 9001.
    loop {
        /*
            Receive OSC data length in var "br". Address of origin data in "a".
            Writes the data received to the buffer on the stack "buf".
        */
        let (br, _a) = sock.recv_from(&mut buf).unwrap();

        /*
            Checks that the packet is greater than 0 bytes.
            If the packet length is <= 0 then the recv loop is restarted.
            The received buffer is then decoded and parsed.
            If the decoded packet "pkt" is of OSC type Message
            the OSC address and OSC args are printed to the CLI.
        */
        if br <= 0 {
            continue;
        } else {
            let pkt = rosc::decoder::decode(&buf).unwrap();
            match pkt {
                OscPacket::Message(msg) => {
                    let addr = msg.addr;
                    let args = msg.args;

                    if addr.eq("/avatar/change") {
                        is_saving = false;
                        avatar_id = match parse_id(args) {
                            Some(av_id) => {
                                println!("\n\n[+] Retrieved avatar ID: {}", &av_id);
                                match ass::load_state(&av_id) {
                                    Some(map) => state = map,
                                    _ => {}
                                };
                                avatar_id
                            }, 
                            None => {
                                println!("\n\n[-] Failed to retrieve avatar ID");
                                String::new()
                            } 
                        };
                        continue;
                    }
                    
                    // haptics boi
                    if addr.contains("Haptics") && !addr.contains("_") {
                        if addr.contains("Right") {
                            remi_ovr.vibrate_controller(TrackedControllerRole::RightHand);
                            self::util::send_data(sock, "avatar/parameters/OSC/Haptics/Right", OscType::Bool(false));
                        } else if addr.contains("Left") {
                            remi_ovr.vibrate_controller(TrackedControllerRole::LeftHand);
                            self::util::send_data(sock, "avatar/parameters/OSC/Haptics/Left", OscType::Bool(false));
                        }
                        continue;
                    }

                    // strip 'avatar/parameters' from the address
                    let avatar_param = &addr[19..];
                    if state.contains_key(avatar_param) {
                        match &args[0] {
                            OscType::Bool(b) => {
                                println!("  <== Received bool {} as {}", &avatar_param, *b);
                                state.insert(avatar_param.to_string(), ass::AssParam::Bool(*b));
                            },
                            OscType::Float(f) => {
                                println!("  <== Received float {} as {}", &avatar_param, *f);
                                state.insert(avatar_param.to_string(), ass::AssParam::Float(*f as f32));
                            },
                            _ => {}
                        }
                        continue;
                    }

                    // set saving
                    if addr.contains("OSC/IsSaving") {
                        match &args[0] {
                            OscType::Bool(b) =>{
                                is_saving = *b;
                                println!("Is Saving? {}", b);
                            },
                            _ => {}
                        }
                        continue;
                    } 

                    // load or save current avatar state
                    if addr.contains("State") {
                        match &args[0] {
                            OscType::Bool(b) => {
                                if *b {
                                    let params = addr.split('/')
                                        .map(|s| s.to_string())
                                        .collect::<Vec<String>>();
            
                                    let state_id = params[params.len()-1].as_str().parse::<i32>().unwrap() as usize;
                                    if is_saving {
                                        ass::save_state(&state, state_id, &ass_dir);
                                    } else {
                                        ass::send_state(&sock, state_id, &ass_dir);
                                    }

                                }
                            },
                            _ => {}
                        }
                        continue;
                    }
                },
                _ => {}
            }
        }
    }
}


fn main() {
    let path = self::ass::initialize_ass_dir().expect("[-] Failed to initialize ASS directory");
    println!("[*] ASS root directory found.");
    
    /*
        Binds/creates a UDP socket to port 9001 to be used for communication with VRChat.
        VRChat binds to UDP port 9000.
    */
    match UdpSocket::bind(format!("127.0.0.1:9001")) {
        Ok(sock) => {
            println!("[*] Remi OSC starting up!");
            let remi_ovr = ovr::RemiOVR::new();
            if remi_ovr.has_context() {
                println!("[++] OpenVR runtime initialized and ready!");
            } else {
                println!("[!!] Failed to initialize OpenVR Runtime - Haptics will not be enabled. Is SteamVR Running?");
            }
            start(&sock, path, remi_ovr);
        },
        Err(_) => {
            println!("[!!] Failed to bind to port 9001 - program will now close.");
        }
    }
}
