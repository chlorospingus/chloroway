use core::str;
use std::{collections::HashMap, string};

use super::shm::ShmPool;

#[derive(Debug)]
enum ParseMode {
    Default,
    Outer,
    Inner,
    Min,
    Max,
    Keysym,
    Keycode,
}
use ParseMode::*;

pub fn gen_id_keysym_mapping(xkb_map: &ShmPool) -> Option<HashMap<u32, String>> {
    let mut word = String::new();
    let mut keycodes: bool = false;

    let mut parse_mode = ParseMode::Default;

    let mut max = 0;
    let mut min = 0;

    let mut keycode: String = String::new();
    let mut keysym: String = String::new();

    let mut res: HashMap<u32, String> = HashMap::new();

    // TODO: Make this not a nightmare if else tree
    for ch in unsafe {str::from_raw_parts(xkb_map.addr as *mut u8, xkb_map.size)}.chars() {
        match parse_mode {
            Default => {
                if ch.is_whitespace() {
                    word.clear();
                } else {
                    word.push(ch);
                }
                if word == "xkb_keycodes" {
                    parse_mode = Outer;
                }
            }
            Outer => {
                if ch == '{' {
                    parse_mode = Inner;
                }
            }
            Inner => {
                if ch.is_whitespace() {
                    word.clear();
                } else {
                    word.push(ch);
                }

                if ch == '}' {
                    break;
                } else if ch == '<' {
                    parse_mode = Keysym;
                } else if ch == '=' {
                    parse_mode = Keycode;
                }

                if word == "minimum" {
                    parse_mode = Min;
                }
                else if word == "maximum" {
                    parse_mode = Max;
                }
            }
            Min => {
                if ch == ';' {
                    min = u32::from_str_radix(keycode.as_str(), 10).unwrap();
                    keycode.clear();
                    parse_mode = Inner;
                } else if ch.is_digit(10) {
                    keycode.push(ch);
                }
            }
            Max => {
                if ch == ';' {
                    max = u32::from_str_radix(keycode.as_str(), 10).unwrap();
                    keycode.clear();
                    parse_mode = Inner;
                } else if ch.is_digit(10) {
                    keycode.push(ch);
                }
            }
            Keysym => {
                if ch == '>' {
                    parse_mode = Inner;
                } else {
                    keysym.push(ch);
                }
            }
            Keycode => {
                if ch == ';' {
                    let code = u32::from_str_radix(keycode.as_str(), 10).unwrap();
                    res.insert(code, keysym.clone());
                    if u32::from_str_radix(keycode.as_str(), 10).unwrap() >= max {
                        break;
                    }
                    keycode.clear();
                    keysym.clear();
                    parse_mode = Inner;
                } else if ch.is_digit(10) {
                    keycode.push(ch);
                }
            }
        }
    }

    Some(res)
}
