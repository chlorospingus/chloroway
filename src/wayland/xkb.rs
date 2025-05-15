use core::str;
use std::{collections::HashMap, fmt::{Debug, Display}, string::{self, ParseError}};

use super::shm::ShmPool;

enum Data<'a> {
    Block(&'a str, Vec<Data<'a>>),
    Statement(&'a str),
}
impl Debug for Data<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Block(header, data) => write!(f, "Block({}, {:#?})", header, data),
            Statement(str) => write!(f, "Statement({})", str),
        }
    }
}
use Data::*;

fn parse_block(map: &str) -> Vec<Data> {
    let mut begin = 0;
    let mut stack = 0;
    let mut is_statement = true;
    let mut datas = Vec::new();
    for (i, byte) in map.bytes().enumerate() {
        if stack == 0 {
            if byte == '{' as u8 {
                datas.push(Block(
                    map.get(begin..i).unwrap(),
                    parse_block(map.get(i+1..).unwrap())
                ));
                is_statement = false;
            }
            else if byte == '}' as u8 {
                return datas;
            }
            else if byte == ';' as u8 || byte == ',' as u8 || byte == ']' as u8 {
                if is_statement {
                    datas.push(Statement(map.get(begin..i).unwrap()));
                } else {
                    is_statement = true;
                }
                begin = i+1;
            }
            else if byte == '[' as u8 {
                begin = i+1;
            }
        }

        if byte == '{' as u8 {
            stack += 1;
        }
        else if byte == '}' as u8 {
            stack -= 1;
        }
    };

    datas
}

fn xkb_symbols(xkb_map: &Data) -> Result<HashMap<String, Vec<String>>, ()> {
    let mut map_symbols: Option<&Vec<Data>> = None;
    if let Block(header, subdata) = xkb_map {
        if header.find("xkb_keymap").is_some() {
            for subdata in subdata {
                if let Block(header, subdata) = subdata {
                    if header.find("xkb_symbols").is_some() {
                        map_symbols = Some(&subdata);
                    }
                }
            }
        }
    }

    let map_symbols: &Vec<Data> = map_symbols.ok_or(())?;
    let mut res = HashMap::new();
    for map_symbols in map_symbols {
        if let Block(header, map_symbols) = map_symbols {
            if header.find("key").is_some() {
                let keysym = header.get(
                    header.find('<').ok_or(())?+1..
                    header.find('>').ok_or(())?
                ).ok_or(())?.to_string();
                let mut symbols = Vec::new();
                for map_symbol in map_symbols {
                    if let Statement(map_symbol) = &map_symbol {
                        let mut symbol = String::new();
                        for ch in map_symbol.chars() {
                            if !ch.is_whitespace() {
                                symbol.push(ch);
                            }
                        }
                        symbols.push(symbol);
                    }
                }
                res.insert(keysym, symbols);
            }
        }
    }

    Ok(res)
}

fn first_non_whitespace(s: &str) -> Option<char> {
    for ch in s.chars() {
        if !ch.is_whitespace() {
            return Some(ch);
        }
    }
    return None;
}

fn xkb_keycodes(xkb_map: &Data) -> Result<HashMap<u32, String>, ()> {
    let mut keycodes: Option<&Vec<Data>> = None;
    if let Block(header, subdata) = xkb_map {
        if header.find("xkb_keymap").is_some() {
            for subdata in subdata {
                if let Block(header, subdata) = subdata {
                    if header.find("xkb_keycodes").is_some() {
                        keycodes = Some(&subdata);
                    }
                }
            }
        }
    }

    let mut res = HashMap::new();
    let keycodes = keycodes.ok_or(())?;
    for map_keycode in keycodes {
        if let Statement(map_keycode) = map_keycode {
            if Some('<') != first_non_whitespace(map_keycode) {
                continue;
            }
            let keysym = map_keycode.get(
                map_keycode.find('<').ok_or(())?+1..
                map_keycode.find('>').ok_or(())?
            ).ok_or(())?;
            let mut keycode = String::new();
            for ch in map_keycode.get(map_keycode.find('=').ok_or(())?..).ok_or(())?.chars() {
                if ch.is_digit(10) {
                    keycode.push(ch);
                }
            }
            let keycode: u32 = keycode.parse().or(Err(()))?;
            res.insert(keycode, keysym.to_string());
        }
    }
    

    Ok(res)
}

pub fn gen_id_keysym_mapping(xkb_map: &ShmPool) -> Option<HashMap<u32, Vec<String>>> {
    let xkb_map = unsafe {str::from_raw_parts(xkb_map.addr as *const u8, xkb_map.size)};
    let mut res = HashMap::new();
    let data = parse_block(xkb_map);
    let mut symbols = xkb_symbols(&data[0]).ok()?;
    let keycodes = xkb_keycodes(&data[0]).ok()?;
    for keycode in keycodes {
        let symbol = symbols.remove(&keycode.1);
        if let Some(symbol) = symbol {
            res.insert(keycode.0, symbol);
        }
    }

    Some(res)
}
