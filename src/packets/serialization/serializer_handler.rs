use std::fmt::{Debug, Display};
use crate::packets::serialization::serialize_error::SerializingErr;

pub type DeserializeResult<'a, T> = Result<(T, &'a [u8]), SerializingErr>;

pub struct McSerializer {
    pub output: Vec<u8>
}

impl McSerializer {
    pub fn new() -> Self {
        Self {
            output: vec![]
        }
    }

    pub fn clear(&mut self) {
        self.output.clear();
    }

    pub fn serialize_bytes(&mut self, input: &[u8]) {
        let mut i = self.output.len();
        self.output.resize(self.output.len() + input.len(), 1); // maybe this is helpful?

        for b in input {
            self.output[i] = *b;
            i += 1;
        }
    }

    pub fn serialize_vec(&mut self, vec: Vec<u8>) {
        self.serialize_bytes(vec.as_slice());
    }

    pub fn serialize_u8(&mut self, b: u8) {
        self.output.push(b);
    }
}

pub trait McDeserialize {
    fn mc_deserialize(input: &mut [u8]) -> DeserializeResult<Self> where Self: Sized;
}

pub trait McSerialize {
    fn mc_serialize(&self, serializer: &mut McSerializer) -> Result<(), SerializingErr>;
}

#[test]
fn serialize_handshake() {
    /*let handshake = v1_20::HandshakingBody {
        protocol_version: VarInt(758),
        server_address: "localhost".to_string(),
        port: 25565,
        next_state: VarInt(1),
    };

    let mut serializer = McSerializer::new();

    handshake.serialize(&mut serializer).unwrap();
    println!("{:?}", serializer.output);*/

    // length, id      protocol      Address                                          port         next state
    // [16, 0,         246, 5,       9, 108, 111, 99, 97, 108, 104, 111, 115, 116,    99, 221,     1]
}