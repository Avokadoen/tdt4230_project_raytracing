use core::fmt;
use std::collections::HashMap;

use cgmath::Vector3;
use ply_reader::{seek_end_of_line, seek_end_word};

use crate::resources::Resources;

/*
    !DO NOT USE THIS AS A REFERENCE FOR LOADING PLY DATA!
    Implemenation for loading a subset the ply format with point based data
*/

#[derive(Debug)]
pub enum ParseError {
    FailedLoading(String),
    Unexpected(String, ReadState, usize),
    InvalidState(ReadState, usize)
    // TODO: parse to type error
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self {
            ParseError::FailedLoading(s) => write!(f, "{}", s),
            ParseError::Unexpected(what, state, offset) => {
                // TODO: we can give better error messages here based on state ...
                write!(f, "Unexpected {} at offset '{}', State: {:?}", what, offset, state)
            },
            ParseError::InvalidState(state, offset) => write!(f, "Parser ended in an invalid state: {:?}, offset: {}", state, offset)
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum ReadState {
    ReadHeader(HeaderSubstate),
    ReadPoint,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum HeaderSubstate {
    Ply,
    Format,
    InferLine,
    Comment,
    Element,
    Propery,
}
// suppoted types and currently expected variables
#[derive(Debug)]
pub enum Identifier {
    X,
    Y,
    Z,
    R,
    G,
    B
}

#[derive(Debug)]
pub enum Type {
    Pos(i32),
    Uchar(u8),
    AlbedoIndex(usize)
}

#[derive(Debug)]
pub struct Variable {
    id: Identifier,
    _type: Type
}

#[derive(Debug)]
pub struct Header {
    pub vertex: usize,
    pub properties: Vec<Variable>
}

#[derive(Debug)]
pub struct PlyVoxel {
    pos: Vector3<i32>,
    albedo_key: u32,
}

#[derive(Debug)]
// File content stored in a way that will help with generating 
// an application specific octree 
pub struct PlyFileContent {
    pub header: Header,
    pub voxels: Vec<PlyVoxel>,
    pub albedos: HashMap<u32, Vector3<u8>>,
    pub scale: f32,
    pub min_point: Vector3<i32>
}

// impl PlyFileContent {
//     pub fn into_vbo(self) -> VertexBufferObject {

//     }
// }

pub fn from_resources(resources: &Resources, name: &str) -> Result<PlyFileContent, ParseError> {
    let buffer = resources.load_buffer(name)
        .map_err(|e| ParseError::FailedLoading(format!("Error loading resource {}: {:?}", name, e)))?;
    
    let mut state = ReadState::ReadHeader(HeaderSubstate::Ply);

    // skip title and version
    let mut offset: usize = 0; 
    let header = {   
        let mut partial_header = Header {
            vertex: 0,
            properties: Vec::<Variable>::with_capacity(6),
        };

        loop {
            match state {
                ReadState::ReadHeader(s) => match s {
                    HeaderSubstate::Ply => {
                        const STRIDE: usize = 5;
                        let mb_err_offset = ply_reader::expect("ply\r\n", &buffer[offset..offset+STRIDE]);
                        if let Some(err_offset) = mb_err_offset {
                            return Err(ParseError::Unexpected(String::from("character"), state, err_offset));
                        }
                        offset += STRIDE;
                        state = ReadState::ReadHeader(HeaderSubstate::Format);
                    },
                    HeaderSubstate::Format => {
                        const STRIDE: usize = 18;
                        let mb_err_offset = ply_reader::expect("format ascii 1.0\r\n", &buffer[offset..offset+STRIDE]); 
                        if let Some(err_offset) = mb_err_offset {
                            return Err(ParseError::Unexpected(String::from("format"), state, err_offset));
                        }
                        offset += STRIDE;
                        state = ReadState::ReadHeader(HeaderSubstate::InferLine);
                    },
                    HeaderSubstate::InferLine => {
                        // TODO: don't unwrap here
                        let end = ply_reader::seek_end_word(&buffer[offset..]).unwrap();
                        // TODO: this part is slow (might not matter as this is the header) 
                        if let None = ply_reader::expect("property", &buffer[offset..offset+end]) { 
                            state = ReadState::ReadHeader(HeaderSubstate::Propery);
                            offset += 9;
                        } else if let None = ply_reader::expect("comment", &buffer[offset..offset+end]) {
                            state = ReadState::ReadHeader(HeaderSubstate::Comment);
                            offset += 8;
                        } else if let None = ply_reader::expect("element", &buffer[offset..offset+end]) {
                            state = ReadState::ReadHeader(HeaderSubstate::Element);
                            offset += 8;
                        } else if let None = ply_reader::expect("end_header", &buffer[offset..offset+end]) {
                            state = ReadState::ReadPoint;
                            offset += 12;
                            break;
                        }
                    },
                    HeaderSubstate::Comment => {
                        // TODO: don't unwrap
                        let end = ply_reader::seek_end_of_line(&buffer[offset..]).unwrap();
                        offset += end + 2;
                        state = ReadState::ReadHeader(HeaderSubstate::InferLine);
                    },
                    HeaderSubstate::Element => {
                        if let Some(err_offset) = ply_reader::expect("vertex", &buffer[offset..]) {
                            return Err(ParseError::Unexpected(String::from("character"), state, err_offset));
                        }
                        offset += 7;
    
                        let end_of_line = seek_end_of_line(&buffer[offset..]).unwrap();
                        
                        partial_header.vertex = match String::from_utf8(buffer[offset..offset+end_of_line].to_vec()) {
                            Err(_e) => return Err(ParseError::Unexpected(String::from("FromUTF8Error"), state, offset)),
                            Ok(value) => value.parse().unwrap(), // TODO: unwanted unwrap
                        };
                        offset += end_of_line + 2;
                        state = ReadState::ReadHeader(HeaderSubstate::InferLine);
                    }
                    HeaderSubstate::Propery => {
                        const X: u8 = 0x78;
                        const Y: u8 = 0x79;
                        const Z: u8 = 0x7A;
                        const R: u8 = 0x72;
                        const G: u8 = 0x67;
                        const B: u8 = 0x62;
    
                        // MagicVoxel use float property as position, this data is also i32/64, not float ...
                        let _type = if let None = ply_reader::expect("float", &buffer[offset..]) {
                            offset += 6;
                            Type::Pos(0)
                        } else if let None = ply_reader::expect("uchar", &buffer[offset..]) {
                            offset += 6;
                            Type::Uchar(0)
                        } else {
                            return Err(ParseError::Unexpected(String::from("type"), state, offset));
                        };
                     
                        let id = match buffer[offset] {
                            X => Identifier::X,
                            Y => Identifier::Y,
                            Z => Identifier::Z,
                            R => Identifier::R,
                            G => Identifier::G,
                            B => Identifier::B,
                            _ => return Err(ParseError::Unexpected(String::from("variable"), state, offset)),
                        };
    
                        partial_header.properties.push(Variable {id, _type});
                        // TODO: unwrap 
                        offset += seek_end_of_line(&buffer[offset..]).unwrap() + 2;
                        state = ReadState::ReadHeader(HeaderSubstate::InferLine);
                    }
                }
                ReadState::ReadPoint => return Err(ParseError::InvalidState(state, offset)),
            }
        }

        partial_header
    };
    
    let lines = header.vertex;
    let mut content = PlyFileContent {
        header,
        voxels: Vec::with_capacity(lines),
        albedos: HashMap::with_capacity(lines / 2),
        scale: 0.0,
        min_point: Vector3::new(i32::max_value(), i32::max_value(), i32::max_value())
    };

    let cantor_pair = |a: Vector3<u8>| -> u32 {
        let fa = a.x as f64;
        let fb = a.y as f64;
        let fc = a.z as f64;

        let internal = |a: f64, b: f64| -> f64 {
            0.5 * (a + b) * (a + b + 1.0) + b
        };

        let fd = internal(fa, fb);
        let hash = 0.5 * (fd + fc) * (fd + fc + 1.0) + fc;

        hash as u32
    };

    loop {
        match state {
            ReadState::ReadPoint => {
                // Test if we are EOF
                match seek_end_of_line(&buffer[offset..]) {
                    Some(_) => {},
                    None => break, // EOF
                }

                let mut voxel = PlyVoxel {
                    pos: Vector3::new(0, 0, 0),
                    albedo_key: 0
                };
                let mut albedo = Vector3::<u8>::new(0, 0, 0);
                for var in  &content.header.properties {
                    let w_end = seek_end_word(&buffer[offset..]).unwrap();
                    let utf8 = String::from_utf8(buffer[offset..offset+w_end].to_vec()).unwrap();
 
                    match var._type {
                        Type::Pos(_) => {
                            let v = match utf8.parse::<i32>() {
                                Err(_e) => return Err(ParseError::Unexpected(String::from("FloatParseError"), state, offset)),
                                Ok(v) => v,
                            };

                            match var.id {
                                Identifier::X => {
                                    content.min_point.x = content.min_point.x.min(v);
                                    voxel.pos.x = v;
                                },
                                Identifier::Y => {
                                    content.min_point.y = content.min_point.y.min(v);
                                    voxel.pos.y = v;
                                },
                                Identifier::Z => {
                                    content.min_point.z = content.min_point.z.min(v);
                                    voxel.pos.z = v;
                                },
                                _ => {}   
                            }
                        },
                        Type::Uchar(_) => {
                            let v = match utf8.parse::<u8>() {
                                Err(_e) => return Err(ParseError::Unexpected(String::from("FloatParseError"), state, offset)),
                                Ok(v) => v,
                            };
                            match var.id {
                                Identifier::R => {
                                    albedo.x = v;
                                },
                                Identifier::G => {
                                    albedo.y = v;
                                },
                                Identifier::B => {
                                    albedo.z = v;
                                },
                                _ => {}   
                            }
                        }
                        Type::AlbedoIndex(_) => return Err(ParseError::Unexpected(String::from("Type::AlbedoIndex"), state, offset))
                    };
                    voxel.albedo_key = cantor_pair(albedo);
                    
                    if !content.albedos.contains_key(&voxel.albedo_key) {
                        content.albedos.insert(voxel.albedo_key, albedo);
                    }

                    offset += w_end + 1;
                }
                content.voxels.push(voxel);
                offset += 1;
            }
            ReadState::ReadHeader(_) => return Err(ParseError::InvalidState(state, offset)),
        }
    }

    Ok(content)
}

mod ply_reader {
    pub const CR: u8 = 0x0D;
    pub const NL: u8 = 0x0A;
    pub const SPACE: u8 = 0x20;

    // TODO: Error, not option
    pub fn expect(expected: &str, bytes: &[u8]) -> Option<usize> {
        let err = expected.chars().zip(bytes.iter()).enumerate().find(|c_b| (c_b.1).0 as u8 != *(c_b.1).1);
        err.map(|elem| elem.0)
    }

    pub fn seek_end_of_line(bytes: &[u8]) -> Option<usize> {
        bytes.iter().enumerate().find(|b| *b.1 == CR || *b.1 == NL).map(|b| b.0)
    }

    pub fn seek_end_word(bytes: &[u8]) -> Option<usize> {
        bytes.iter().enumerate().find(|b| *b.1 == SPACE || *b.1 == CR || *b.1 == NL).map(|b| b.0)
    }
}
