use base64::{alphabet, Engine};
use base64::alphabet::Alphabet;
use base64::engine::{GeneralPurpose, GeneralPurposeConfig};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::packets::packet_definer::{PacketState, PacketTrait};
use crate::packets::serialization::serializer_error::SerializingErr;
use crate::packets::serialization::serializer_handler::{DeserializeResult, McDeserialize, McDeserializer, McSerialize, McSerializer, StateBasedDeserializer};
use crate::protocol_details::datatypes::var_types::VarInt;
use crate::protocol_details::protocol_verison::ProtocolVerison;

const ALPHABET: Alphabet = alphabet::STANDARD;
const CONFIG: GeneralPurposeConfig = GeneralPurposeConfig::new();
const ENGINE: GeneralPurpose = GeneralPurpose::new(&ALPHABET, CONFIG);

#[derive(Debug, Clone)]
pub struct UniversalHandshakePacket {
	pub protocol_version: VarInt,
	pub server_address: String,
	pub server_port: u16,
	pub next_state: VarInt,
}

impl UniversalHandshakePacket {
	pub fn new(protocol_version: VarInt, server_address: String, server_port: u16, next_state: VarInt) -> Self {
		Self {
			protocol_version,
			server_address,
			server_port,
			next_state,
		}
	}
}

impl McSerialize for UniversalHandshakePacket {
	fn mc_serialize(&self, serializer: &mut McSerializer) -> Result<(), SerializingErr> {
		self.protocol_version.mc_serialize(serializer)?;
		self.server_address.mc_serialize(serializer)?;
		self.server_port.mc_serialize(serializer)?;
		self.next_state.mc_serialize(serializer)?;

		Ok(())
	}
}

impl StateBasedDeserializer for UniversalHandshakePacket {
	fn deserialize_state<'a>(deserializer: &'a mut McDeserializer, state: &PacketState) -> DeserializeResult<'a, Self> {
		if state != &PacketState::HANDSHAKING {
			return Err(SerializingErr::InvalidPacketState);
		}

		let raw = UniversalHandshakePacket {
			protocol_version: VarInt::mc_deserialize(deserializer)?,
			server_address: String::mc_deserialize(deserializer)?,
			server_port: u16::mc_deserialize(deserializer)?,
			next_state: VarInt::mc_deserialize(deserializer)?,
		};

		Ok(raw)
	}
}

impl PacketTrait for UniversalHandshakePacket {
	fn packet_id() -> u8 {
		0x00
	}

	fn state() -> PacketState {
		PacketState::HANDSHAKING
	}
}

#[derive(Debug, Clone)]
pub struct UniversalStatusRequest;

impl McSerialize for UniversalStatusRequest {
	fn mc_serialize(&self, _serializer: &mut McSerializer) -> Result<(), SerializingErr> {
		Ok(())
	}
}

impl StateBasedDeserializer for UniversalStatusRequest {
	fn deserialize_state<'a>(deserializer: &'a mut McDeserializer, state: &PacketState) -> DeserializeResult<'a, Self> {
		if state != &PacketState::STATUS {
			return Err(SerializingErr::InvalidPacketState);
		}

		Ok(UniversalStatusRequest)
	}
}

impl PacketTrait for UniversalStatusRequest {
	fn packet_id() -> u8 {
		0x00
	}

	fn state() -> PacketState {
		PacketState::STATUS
	}
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(non_snake_case)]
pub struct UniversalStatusResponse {
	version: VersionInfo,
	players: PlayerInfo,
	description: DescriptionInfo,
	#[serde(skip_serializing_if = "Option::is_none")]
	favicon: Option<String>,
	enforcesSecureChat: bool,
	previewsChat: bool,
}

impl UniversalStatusResponse {
	pub fn new(protocol_verison: ProtocolVerison, description: String) -> Self {
		Self {
			version: VersionInfo {
				name: protocol_verison.get_fancy_name(),
				protocol: protocol_verison.get_version_number(),
			},
			players: PlayerInfo {
				max: 0,
				online: 0,
				sample: Vec::new(),
			},
			description: DescriptionInfo {
				text: description
			},
			favicon: None,
			enforcesSecureChat: false,
			previewsChat: false,
		}
	}

	/// The server icon should be a 64x64 PNG image, without new line (\n) characters.
	pub fn set_favicon(&mut self, data: Option<&[u8]>) {
		if let Some(data) = data {
			let mut s = "data:image/png;base64,".to_string();

			s.push_str(&ENGINE.encode(data));

			self.favicon = Some(s);
		} else {
			self.favicon = None;
		}
	}

	pub fn set_secure_chat(&mut self, secure: bool) {
		self.enforcesSecureChat = secure;
	}

	pub fn set_preview_chat(&mut self, preview: bool) {
		self.previewsChat = preview;
	}

	pub fn set_description(&mut self, description: String) {
		self.description = DescriptionInfo {
			text: description
		};
	}

	/// *version* can really be anything you want, but *protocol_version* must be a valid protocol version number
	pub fn set_protocol_version(&mut self, version: String, protocol_version: i16) {
		self.version = VersionInfo {
			name: version,
			protocol: protocol_version,
		};
	}
}

impl McSerialize for UniversalStatusResponse {
	fn mc_serialize(&self, serializer: &mut McSerializer) -> Result<(), SerializingErr> {
		let serialized = serde_json::to_string(self).unwrap();

		serialized.mc_serialize(serializer)?;

		Ok(())
	}
}

impl StateBasedDeserializer for UniversalStatusResponse {
	fn deserialize_state<'a>(deserializer: &'a mut McDeserializer, state: &PacketState) -> DeserializeResult<'a, Self> {
		if state != &PacketState::STATUS {
			return Err(SerializingErr::InvalidPacketState);
		}

		let raw = serde_json::from_str(&String::mc_deserialize(deserializer)?).unwrap(); // TODO: test

		Ok(raw)
	}
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionInfo {
	name: String,
	protocol: i16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerInfo {
	max: i32,
	online: i32,
	sample: Vec<PlayerSample>,
}

impl PlayerInfo {
	pub fn add_player(&mut self, player: PlayerSample) {
		self.sample.push(player);
	}

	pub fn set_player_sample(&mut self, players: Vec<PlayerSample>) {
		self.sample = players;
	}
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DescriptionInfo { // TODO: update to chat thing?
	text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerSample {
	name: String,
	id: String,
}

impl PlayerSample {
	pub fn new(name: String, id: Uuid) -> Self {
		Self {
			name,
			id: id.to_string(),
		}
	}
}

#[derive(Debug, Clone)]
pub struct UniversalPingRequest {
	pub payload: i64,
}

impl McSerialize for UniversalPingRequest {
	fn mc_serialize(&self, serializer: &mut McSerializer) -> Result<(), SerializingErr> {
		self.payload.mc_serialize(serializer)?;

		Ok(())
	}
}

impl StateBasedDeserializer for UniversalPingRequest {
	fn deserialize_state<'a>(deserializer: &'a mut McDeserializer, state: &PacketState) -> DeserializeResult<'a, Self> {
		if state != &PacketState::STATUS {
			return Err(SerializingErr::InvalidPacketState);
		}

		let raw = UniversalPingRequest {
			payload: i64::mc_deserialize(deserializer)?,
		};

		Ok(raw)
	}
}

#[derive(Debug, Clone)]
pub struct UniversalPingResponse {
	pub payload: i64,
}

impl McSerialize for UniversalPingResponse {
	fn mc_serialize(&self, serializer: &mut McSerializer) -> Result<(), SerializingErr> {
		self.payload.mc_serialize(serializer)?;

		Ok(())
	}
}

impl StateBasedDeserializer for UniversalPingResponse {
	fn deserialize_state<'a>(deserializer: &'a mut McDeserializer, state: &PacketState) -> DeserializeResult<'a, Self> {
		if state != &PacketState::STATUS {
			return Err(SerializingErr::InvalidPacketState);
		}

		let raw = UniversalPingResponse {
			payload: i64::mc_deserialize(deserializer)?,
		};

		Ok(raw)
	}
}

