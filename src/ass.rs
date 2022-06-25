// Rust OSC library import
extern crate rosc;

use rosc::OscType;
// Serde
use serde::{Serialize, Deserialize};
//use serde_json::{Serializer, Deserializer, to_string, Value};
use serde_json::{self, Value};
use std::net::UdpSocket;
// IPv4 Address / Udp Socket object
use std::{fs, f32};

// Hash Map
use std::collections::HashMap;

// Utils
#[path = "utils.rs"] mod util;

#[derive(Debug, Deserialize, Serialize)]
pub enum AssParam {
    Bool(bool),
    Float(f32),
}

pub fn get_ass_dir() -> String {
    return format!(
        "{}\\AppData\\LocalLow\\VRChat\\VRChat\\OSC\\ASS\\States", 
        self::util::get_user_home_dir()
    ).to_string();
}

pub fn get_facetracking_fix_dir() -> String {
    return format!(
        "{}\\AppData\\LocalLow\\VRChat\\VRChat\\OSC\\ASS\\Avatars", 
        self::util::get_user_home_dir()
    ).to_string();
}

pub fn initialize_ass_dir() -> Result<String, String> {
    let ass_dir = get_ass_dir();
    if !self::util::dir_exists(&ass_dir) {
        match fs::create_dir_all(&ass_dir) {
            Ok(_) => return Ok(ass_dir),
            Err(_) => return Err("Failed to create Ass root directory.".to_string())
        }
    } else {
        println!("[*] ASS root directory found!");
    }

    let ft_fix_dir = get_facetracking_fix_dir();
    if !self::util::dir_exists(&ft_fix_dir) {
        match fs::create_dir_all(&ft_fix_dir) {
            Ok(_) => return Ok(ft_fix_dir),
            Err(_) => return Err("Failed to create Avatars directory.".to_string())
        }
    }

    return Ok(ass_dir);
}

pub fn new_parameter_list() -> HashMap<String, AssParam> {
    return HashMap::from([
        ("Toggles/SwimsuitOn".to_string(), AssParam::Bool(true)),
        ("Toggles/JacketOn".to_string(), AssParam::Bool(true)),
        ("Toggles/Tops/CropOn".to_string(), AssParam::Bool(false)),
        ("Toggles/Tops/SportsBraOn".to_string(), AssParam::Bool(false)),
        ("Toggles/Tops/TankTopOn".to_string(), AssParam::Bool(true)),
        ("Toggles/Tops/NSFW".to_string(), AssParam::Bool(false)),
        ("Toggles/Acc/BeanieOff".to_string(), AssParam::Bool(false)),
        ("Toggles/Acc/GlovesOff".to_string(), AssParam::Bool(false)),
        ("Toggles/Acc/HeadphonesOff".to_string(), AssParam::Bool(false)),
        ("Toggles/Acc/MaskOff".to_string(), AssParam::Bool(false)),
        ("Toggles/Acc/ThighSocksOn".to_string(), AssParam::Bool(true)),
        ("Toggles/Acc/CalfSocksOn".to_string(), AssParam::Bool(false)),
        ("Toggles/Acc/SleevesOff".to_string(), AssParam::Bool(false)),
        ("Toggles/Acc/HolotagArmOff".to_string(), AssParam::Bool(false)),
        ("Toggles/Acc/HolotagThighOff".to_string(), AssParam::Bool(false)),
        ("Toggles/Acc/RunningShoeOff".to_string(), AssParam::Bool(false)),
        ("Toggles/Bottoms/PantiesOn".to_string(), AssParam::Bool(false)),
        ("Toggles/Bottoms/AssPantsOn".to_string(), AssParam::Bool(true)),
        ("Toggles/Bottoms/PantsOn".to_string(), AssParam::Bool(false)),
        ("Toggles/Bottoms/NSFW".to_string(), AssParam::Bool(false)),
        ("Toggles/Acc/GlassesOff".to_string(), AssParam::Bool(false)),
        ("Toggles/Acc/BackpackOff".to_string(), AssParam::Bool(false)),
        ("Toggles/Acc/DogtagsOff".to_string(), AssParam::Bool(false)),
        ("Toggles/Acc/LipRingsOff".to_string(), AssParam::Bool(false)),
        ("Toggles/Acc/EarringsOff".to_string(), AssParam::Bool(false)),
        ("Hair/ShortOn".to_string(), AssParam::Bool(false)),
        ("Hair/PonyOn".to_string(), AssParam::Bool(false)),
        ("Hair/LongOn".to_string(), AssParam::Bool(true)),
        ("Nails/Normal".to_string(), AssParam::Bool(false)),
        ("Nails/Square".to_string(), AssParam::Bool(false)),
        ("Nails/Almond".to_string(), AssParam::Bool(false)),
        ("Nails/Longer".to_string(), AssParam::Bool(false)),
        ("Nails/Stilletto".to_string(), AssParam::Bool(true)),
        ("Mat/Hue".to_string(), AssParam::Float(0.)),
        ("Mat/Colors/HairHue".to_string(), AssParam::Float(0.)),
        ("Mat/Colors/HairColored".to_string(), AssParam::Bool(true)),
        ("Mat/Cyberpunk".to_string(), AssParam::Bool(true)),
        ("Mat/HairBlack".to_string(), AssParam::Bool(false)),
        ("Mat/HairWhite".to_string(), AssParam::Bool(false)),
        ("Mat/HairBlonde".to_string(), AssParam::Bool(true)),
        ("Mat/Monochromatic".to_string(), AssParam::Bool(false)),
        ("Mat/AudioLink".to_string(), AssParam::Bool(false)),
        ("Mat/Colors/AssPantsDark".to_string(), AssParam::Bool(true)),
        ("Mat/Colors/BackpackDark".to_string(), AssParam::Bool(false)),
        ("Mat/Colors/BeanieDark".to_string(), AssParam::Bool(true)),
        ("Mat/Colors/GlovesDark".to_string(), AssParam::Bool(true)),
        ("Mat/Colors/HeadphonesDark".to_string(), AssParam::Bool(true)),
        ("Mat/Colors/JacketDark".to_string(), AssParam::Bool(false)),
        ("Mat/Colors/MaskDark".to_string(), AssParam::Bool(true)),
        ("Mat/Colors/ShoeDark".to_string(), AssParam::Bool(false)),
        ("Mat/Colors/SleevesDark".to_string(), AssParam::Bool(true)),
        ("Mat/Colors/SwimsuitDark".to_string(), AssParam::Bool(false)),
        ("Mat/Colors/PantsDark".to_string(), AssParam::Bool(false)),
        ("Mat/Colors/SportsBraDark".to_string(), AssParam::Bool(true)),
        ("Mat/Colors/TankTopDark".to_string(), AssParam::Bool(true)),
        ("Mat/Colors/CropDark".to_string(), AssParam::Bool(true)),
    ]);
}

/*
    Converts the VRC Local Avatar Data file into a HashMap that is readable
    by this program - filters out unwanted fields to only those present
    in the parameter list defined in `new_parameter_list()`
*/
fn vrc_state_to_map(state: Value) -> HashMap<String, AssParam> {
    let mut map = new_parameter_list();
    for parameters in state["animationParameters"].as_array() {
        for parameter in parameters {
            let name = parameter["name"].to_string().replace("\"", "");
            let val = &parameter["value"];

            // println!("\tNAME: {:?}", name);
            //assert_eq!(map.contains_key("Nails/Almond"), true);

            if map.contains_key(&name) {
                //println!("[++] Key {} found! Value: {}", name, val);
                if name.contains("Hue") {
                    let val = val.as_f64().unwrap_or(0.);
                    map.insert(name, AssParam::Float(val as f32));
                } else {
                    let val = val.as_i64().unwrap_or(0);
                    map.insert(name, AssParam::Bool(val == 1));
                }
            } else {
            //    println!("D: Key {} not found", &name);
            }
        }
    }

    return map;
}

/*
    Attempts to retrieve avatar config file from {avatar_id} located at:
    C:\\Users\\{username}\\AppData\\LocalLow\\VRChat\\VRChat\\LocalAvatarData\\{user_id}\\{avatar_id}

    If file does not exist or is invalid format return None
    If file exists return AvatarState as retrieved from the file
 */
pub fn load_state(avatar_id: &str) -> Option<HashMap<String, AssParam>> {
    let avatar_data_dir = format!("{}\\AppData\\LocalLow\\VRChat\\VRChat\\LocalAvatarData", util::get_user_home_dir());
    let dirs = fs::read_dir(&avatar_data_dir).expect(format!("Fail to read dir {}", &avatar_data_dir).as_str());
    for dir in dirs {
        //println!("Dir: {:?}", dir);
        match dir {
            Ok(entry) => {
                let avatar_id_path = format!("{}\\{}", entry.path().to_str().unwrap(), &avatar_id);
                let file = fs::read_to_string(&avatar_id_path).unwrap_or_default();
                match serde_json::from_str::<Value>(&file) {
                    Ok(state) => {
                        println!("[*] Found file {:?}", &avatar_id_path);
                        return Some(vrc_state_to_map(state));
                    },
                    Err(e) => {
                        println!("[-] Error loading state from file: {:?}\n    Using default parameter list", e);
                        return Some(new_parameter_list());
                    }
                }
            },
            Err(e) => {
                println!("Incorrect directory...\n{:?}", e);
            }
        }
    }

    return None;
}

/*
    Saves the avatar's state to the ASS directly as given
*/
pub fn save_state(state: &HashMap<String, AssParam>, save_index: usize, ass_dir: &str) {
    let path = format!("{}\\state_{}.json", &ass_dir, save_index.to_string());
    println!("SAVING TO {}", &path);

    match serde_json::to_string(&state) {
        Ok(state_str) => {
            match fs::write(&path, state_str) {
                Ok(_) => println!("[+] Saved state to {} successfully", &path),
                _ => println!("[-] Failed to save the avatar state")
            }
        },
        Err(_) => println!("[!!] Failed to convert avatar state to json")
    }
}

pub fn send_state(sock: &UdpSocket, load_index: usize, ass_dir: &str) {
    let path = format!("{}\\state_{}.json", &ass_dir, load_index.to_string());

    let file = fs::read_to_string(&path).unwrap_or_default();
    match serde_json::from_str::<Value>(&file) {
        Ok(state) => {
            println!("[*] State Transmission Begin");
            let state_obj = state.as_object().unwrap();

            let swimsuit_on: bool = is_swimsuit_on(&state_obj);
            if swimsuit_on {
                self::util::send_data(sock, "Toggles/SwimsuitOn", OscType::Bool(true));
            }

            for param_name in state_obj.keys() {
                let ass_param = AssParam::deserialize(state_obj.get(param_name).unwrap()).unwrap();

                /* 
                    Remi's animator uses integer emulation with booleans for her mutually
                    exclusive toggles; I only want to send the boolean value cooresponding
                    to the item that should be toggled ON, and my animator will take
                    care of the rest.
                */
                if (param_name.contains("Top") || param_name.contains("Bot")) && swimsuit_on {
                    println!("Skipping {} - Swimsuit is on", param_name);
                    continue;
                } 

                if param_name.contains("Top") && !swimsuit_on   {
                    match ass_param {
                        AssParam::Bool(b) => {
                            if b {
                                self::util::send_data(sock, param_name, OscType::Bool(b));
                            }
                        }, _ => {}
                    }
                    continue;
                }
                
                if param_name.contains("Bot") && !swimsuit_on {
                    match ass_param {
                        AssParam::Bool(b) => {
                            if b {
                                self::util::send_data(sock, param_name, OscType::Bool(b));
                            }
                        }, _ => {}
                    }
                    continue;
                }

                if !param_name.contains("Toggles/Swimsuit") {
                    match ass_param {
                        AssParam::Bool(b) => {
                            self::util::send_data(sock, param_name, OscType::Bool(b));
                        },
                        AssParam::Float(f) => {
                            self::util::send_data(sock, param_name, OscType::Float(f as f32));
                        }
                    }
                }
            }
            println!("[*] State Transmission End");
        },
        Err(e) => {
            println!("\n[!!] Error Loading State file: {:?}", e);
        }
    }
}

fn is_swimsuit_on(state_obj: &serde_json::Map<String, Value>) -> bool {
    match state_obj.get("Toggles/SwimsuitOn") {
        Some(obj) => {
            match AssParam::deserialize(obj).unwrap_or(AssParam::Bool(false)) {
                AssParam::Bool(b) => return b,
                _ => return false
            }
        },
        _ => return false
    };
}
