// Cam wants a pretty camel case name
#![allow(non_snake_case)]
// Rust OSC library import
extern crate rosc;

//use ass::load_state;
use serde::{Deserialize, Serialize};
//use ass::{AssParam, send_state};
// Serde
//use serde::{Serialize, Deserialize};
//use serde_json::{Serializer, Deserializer, to_string, Value};
use serde_json;
// ROSC encoder import
//use rosc::encoder;
// ROSC types (Message types / Packet types / ROSC arg types
use rosc::{OscPacket, OscType};
use std::collections::HashMap;
use std::fs;
// IPv4 Address / Udp Socket object
use std::net::UdpSocket;
// From String Trait
//use std::str::FromStr;
// Duration struct
//use std::time::Duration;
// Proc Environment / 32 bit float / thread lib
//use std::io::{Write, Read};
use std::f32;
//use std::path::Path;
//use directories::BaseDirs;

// OpenVR
#[path = "ovr.rs"] mod ovr;
use openvr::TrackedControllerRole;

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
/*
#[allow(dead_code)]
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
}*/


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

    println!("[++] Remi OSC up and running!");

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


#[derive(Serialize, Deserialize, Debug)]
struct OSCOptions {
    bind_host: String,
    bind_port: String,
}

impl Default for OSCOptions {
    fn default() -> Self {
        OSCOptions { bind_host: "127.0.0.1".to_string(), bind_port: "9001".to_string() }
    }
}

impl OSCOptions {
    fn default_config_write(path: &String) -> bool {
        let json_data = match serde_json::to_string(&OSCOptions::default()) {
            Ok(jd) => jd,
            Err(e) => {println!("[!] Failed to serialize OSCOptions default! | Err: {}", e);return false;}
        };

        match fs::write(
            path, 
            json_data
        ) {
            Ok(_) => return true,
            Err(e) => {println!("[!] Failed to write OSCOptions default! | Err: {}", e); return false;}
        }
    }

    fn read_config(path: &String) -> Self {
        match fs::read_to_string(path) {
            Ok(nc) => match serde_json::from_str(&nc) {
                Ok(net_config) => {
                    return net_config;
                },
                Err(e) => {
                    println!("[!] Failed to serialize NetConfig! | Err: {}", e);
                    println!("[!] Continuing with default config! No config will be saved!");
                    return OSCOptions::default();
                }
            },
            Err(e) => {
                println!("[!] Failed to read NetConfig file! | Err: {}", e);
                println!("[!] Continuing with default config! No config will be saved!");
                return OSCOptions::default();
            }
        }
    }
}

fn load_net_config() -> OSCOptions {

    let config_dir = format!("{}\\AppData\\LocalLow\\VRChat\\VRChat\\OSC\\ASS\\NetConfig", util::get_user_home_dir());
    let net_conf_file = format!("{}\\NetConfig.json", config_dir);

    if !util::dir_exists(&config_dir) {
    
        if let Ok(()) = fs::create_dir_all(config_dir) {

            println!("[+] Created NetConfig dir!");
    
            if OSCOptions::default_config_write(&net_conf_file) {
                println!("[+] Wrote default NetConfig!");
            } else {
                println!("[!] Failed to write default NetConfig!");
            }
    
        } else {
            println!("[+] OSC NetConfig dir exists.");

            if !util::file_exists(&net_conf_file) {
                if OSCOptions::default_config_write(&net_conf_file) {
                    println!("[+] Wrote default NetConfig!");
                } else {
                    println!("[!] Failed to write default NetConfig!");
                }
            }
        }
    } else {

        if !util::file_exists(&net_conf_file) {

            if OSCOptions::default_config_write(&net_conf_file) {
                println!("[+] Wrote default NetConfig!");
            } else {
                println!("[!] Failed to write default NetConfig!");
            }
        }
    }

    return OSCOptions::read_config(&net_conf_file);

}

fn main() {
    let path = self::ass::initialize_ass_dir().expect("[-] Failed to initialize Remi-OSC directory");
    
    // Load Network Configuration
    let net_config = load_net_config();

    // Load state Tests
    //let _ = load_state("avtr_d9201c0d-667d-4c0d-8bc4-d379068afa36");
    //let _ = load_state("avtr_7fe42546-7e40-4382-9db6-9a83677e17ee");
    /*
        Binds/creates a UDP socket to port 9001 to be used for communication with VRChat.
        VRChat binds to UDP port 9000.
    */
    match UdpSocket::bind(format!("{}:{}", net_config.bind_host, net_config.bind_port)) {
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
