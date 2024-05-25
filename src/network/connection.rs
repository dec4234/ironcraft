use std::cmp::PartialEq;
use std::fmt::Display;
use std::net::SocketAddr;

use anyhow::{Error, Result};
use log::{debug, trace};
use serde::__private::ser::constrain;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

use crate::network::network_error::{ConnectionAbortedLocally, InvalidPacketState, NoDataReceivedError};
use crate::network::network_structure::LoginHandler;
use crate::packets::packet_definer::PacketState;
use crate::packets::raw_packet::PackagedPacket;
use crate::packets::serialization::serializer_handler::{McDeserialize, McDeserializer, McSerialize, McSerializer, StateBasedDeserializer};
use crate::packets::status::status_handler::StatusHandler;
use crate::packets::status::status_packets::UniversalHandshakePacket;
use crate::protocol_details::datatypes::var_types::VarInt;
use crate::protocol_details::protocol_verison::ProtocolVerison;

const BUFFER_SIZE: usize = 1024;

pub struct CraftClient {
	tcp_stream: TcpStream,
	socket_addr: SocketAddr,
	pub packet_state: PacketState,
	compression_threshold: Option<i32>,
	buffer: Vec<u8>,
	client_version: Option<VarInt>
}

impl CraftClient {
	pub fn from_connection(tcp_stream: TcpStream) -> Result<Self> {
		tcp_stream.set_nodelay(true)?; // disable Nagle's algorithm
		
		Ok(Self {
			socket_addr: tcp_stream.peer_addr()?,
			tcp_stream,
			packet_state: PacketState::HANDSHAKING,
			compression_threshold: None,
			buffer: vec![],
			client_version: None
		})
	}
	
	pub async fn send_packet<P: McSerialize + StateBasedDeserializer>(&mut self, packet: PackagedPacket<P>) -> Result<()> {
		let mut serializer = McSerializer::new();
		packet.mc_serialize(&mut serializer)?;
		let output = &serializer.output;
		
		// TODO: compress & encrypt here
		
		self.tcp_stream.write_all(output).await?;
		Ok(())
	}
	
	// TODO: could use a good optimization pass - reduce # of copies, ideally to 0
	/// Receive a minecraft packet from the client. This will block until a packet is received.
	pub async fn receive_packet<P: McSerialize + StateBasedDeserializer>(&mut self) -> Result<PackagedPacket<P>> {
		if !self.buffer.is_empty() {
			let mut deserializer = McDeserializer::new(&self.buffer);
			let packet = PackagedPacket::deserialize_state(&mut deserializer, &self.packet_state)?;
			self.buffer = deserializer.collect_remaining().to_vec();
			return Ok(packet);
		}

		// TODO: test packets greater than buffer size - just make it really small
		let mut aggregate = vec![];
		let mut agg_length = 0;
		let mut buffer = vec![0; BUFFER_SIZE];
		let length = self.tcp_stream.read(&mut buffer).await;
		
		if let Err(e) = length {
			if e.to_string().contains("An established connection was aborted by the software in your host machine") {
				debug!("OS Error detected in packet receive, closing the connection: {}", e);
				self.close().await;
				return Err(Error::from(ConnectionAbortedLocally));
			}
			
			return Err(Error::from(e));
		}
		
		let length = length.unwrap();
		
		aggregate.append(&mut buffer[0..length].to_vec());
		
		if length == BUFFER_SIZE {
			loop { // TODO: also break at max packet size
				if let Ok(length) = self.tcp_stream.try_read(&mut buffer) {
					if length == 0 {
						break;
					}
					
					agg_length += length;
					aggregate.append(&mut buffer[0..length].to_vec());
					
					if length < BUFFER_SIZE {
						break;
					}
				} else {
					break;
				}
			}
		} else {
			agg_length += length;
		}
		
		trace!("Received {:?}", &buffer[0..length]);

		if length == 0 { // connection closed
			self.close().await;
			return Err(Error::from(NoDataReceivedError));
		}
		
		// TODO: decompress & decrypt here
		
		let mut deserializer = McDeserializer::new(&aggregate[0..agg_length]);
		let packet = PackagedPacket::deserialize_state(&mut deserializer, &self.packet_state)?;

		self.buffer.append(&mut deserializer.collect_remaining().to_vec()); // if the next packet was also collected

		Ok(packet)
	}
	
	pub fn change_state(&mut self, state: PacketState) {
		self.packet_state = state;
	}
	
	// TODO: this won't work with compression, although I think we only use it for the length anyways
	pub async fn peek_next_packet_details(&mut self) -> Result<(VarInt, VarInt)> {
		if !self.buffer.is_empty() {
			let mut deserializer = McDeserializer::new(&self.buffer);
			let length = VarInt::mc_deserialize(&mut deserializer)?;
			let packet_id = VarInt::mc_deserialize(&mut deserializer)?;
			return Ok((length, packet_id));
		}

		let mut buffer = vec![0; BUFFER_SIZE];
		let length = self.tcp_stream.peek(&mut buffer).await?;
		
		if length == 0 {
			return Err(anyhow::anyhow!("No data received"));
		}
		
		let mut deserializer = McDeserializer::new(&buffer[0..length]);
		let length = VarInt::mc_deserialize(&mut deserializer)?;
		let packet_id = VarInt::mc_deserialize(&mut deserializer)?;
		Ok((length, packet_id))
	}
	
	pub fn enable_compression(&mut self, threshold: Option<i32>) {
		self.compression_threshold = threshold;
	}
	
	pub async fn close(&mut self) -> bool {
		debug!("Closing connection to {}", self);
		self.tcp_stream.shutdown().await.is_ok()
	}
	
	/// Get the protocol version of this client as a `ProtocolVersion` enum. This will return 'None' if the 
	/// handshake has not been performed or if the protocol version number is not known to the library
	pub fn get_client_version(&self) -> Option<ProtocolVerison> {
		Some(ProtocolVerison::from(self.client_version?.0 as i16)?)
	}
}

impl Display for CraftClient {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		let s = if let Ok(addr) = self.tcp_stream.peer_addr() {
			format!("{}", addr)
		} else {
			"Unknown".to_string()
		};

		write!(f, "{}", format!("CraftConnection: {}", s))
	}
}