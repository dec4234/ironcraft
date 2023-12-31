use std::fmt::{Debug, Display, Error, Formatter};
use serde::{ser, Serialize, Serializer};
use crate::packets::packet_definer::PacketTrait;
use crate::protocol_details::datatypes::var_types::{VarInt};
use anyhow::Result;
use crate::packets::serialization::serializer_error::SerializingErr;
use crate::packets::serialization::serializer_handler::{DeserializeResult, McDeserialize, McDeserializer, McSerialize, McSerializer};

pub struct RawPacket<P: McSerialize + McDeserialize> {
    length: VarInt,
    packet_id: VarInt,
    data: P
}

impl<P: McSerialize + McDeserialize> McSerialize for RawPacket<P> {
    fn mc_serialize(&self, serializer: &mut McSerializer) -> Result<(), SerializingErr> {
        self.length.mc_serialize(serializer)?;
        self.packet_id.mc_serialize(serializer)?;
        self.data.mc_serialize(serializer)?;

        Ok(())
    }
}

impl<P: McSerialize + McDeserialize> McDeserialize for RawPacket<P> {
    fn mc_deserialize<'a>(deserializer: &'a mut McDeserializer) -> DeserializeResult<'a, Self> {
        let raw = RawPacket {
            length: VarInt::mc_deserialize(deserializer)?,
            packet_id: VarInt::mc_deserialize(deserializer)?,
            data: P::mc_deserialize(deserializer)?,
        };

        return Ok(raw);
    }
}