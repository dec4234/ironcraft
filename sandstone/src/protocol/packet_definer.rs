/*
Defines key macros, traits and enums used to describe packets.
 */

/// Defines the DESTINATION of the packet. So a packet that is C -> S would be `PacketDirection::SERVER`
#[derive(Debug, Clone, PartialEq, Eq, Hash, Copy)]
pub enum PacketDirection {
	SERVER,
	CLIENT,
	BIDIRECTIONAL // are there any?
}

/// Used to help discern the type of packet being received. Note that different states could have
/// packets with the same ids. 
#[derive(Debug, Clone, PartialEq, Eq, Hash, Copy)]
pub enum PacketState {
	STATUS,
	HANDSHAKING,
	LOGIN,
    CONFIGURATION,
	PLAY
}

impl PacketState {
    pub fn from_id(id: u8) -> Option<PacketState> {
        match id {
            1 => Some(PacketState::STATUS),
            2 => Some(PacketState::LOGIN),
            _ => None // others are unknown at this time
        }
    }
    
    pub fn get_id(&self) -> Option<u8> {
        match self {
            PacketState::STATUS => Some(1),
            PacketState::LOGIN => Some(2),
            _ => None
        }
    }
}

#[macro_use]
mod macros {
    /// Used to define the minecraft packet protocol. This includes, the name, packet ID, state and
    /// the respective fields for the packet.
    #[macro_export]
    macro_rules! packets {
        ($ref_ver: ident => {
            // These are split into multiple levels to allow for more efficient deserialization 
            $($state: ident => {
                $($direction: ident => {
                   $($name: ident, $name_body: ident, $packetID: literal => {
                        $($field: ident: $t: ty),*
                    }),* 
                }),*
            }),*
        }) => {
            $(
                $(
                    $(
                        #[derive(Debug, Clone, PartialEq, Eq)]
                        pub struct $name_body { // The body struct of the packet
                            $(pub(crate) $field: $t),*
                        }
                        
                        impl $name_body {
                            pub fn new($($field: $t),*) -> Self {
                                Self {
                                    $($field),*
                                }
                            }
                        }
                    
                        #[allow(unused)] // incase there's an empty packet
                        impl McDeserialize for $name_body {
                            fn mc_deserialize<'a>(deserializer: &'a mut McDeserializer) -> SerializingResult<'a, Self> {
                                let s = Self {
                                    $($field: <$t>::mc_deserialize(deserializer)?,)*
                                };
        
                                Ok(s)
                            }
                        }
                    
                        #[allow(unused)] // incase there's an empty packet
                        impl McSerialize for $name_body {
                            fn mc_serialize(&self, serializer: &mut McSerializer) -> SerializingResult<()> {
                                $(self.$field.mc_serialize(serializer)?;)*
        
                                Ok(())
                            }
                        }
                    
                        impl From<$name_body> for Packet {
                            fn from(p: $name_body) -> Self {
                                Packet::$name(p)
                            }
                        }
                    
                        impl From<Packet> for $name_body {
                            fn from(p: Packet) -> Self {
                                match p {
                                    Packet::$name(p) => p,
                                    _ => panic!("Invalid conversion")
                                }
                            }
                        }
                    )*
                )*
            )*
            
            $crate::as_item!( // weird workaround from mcproto-rs
                #[derive(Debug, Clone, PartialEq, Eq)]
                pub enum Packet {
                    $($($($name($name_body),)*)*)*
                }
            );
            
            impl Packet {
                pub fn packet_id(&self) -> VarInt {
                    match self {
                        $($($(Packet::$name(_) => VarInt($packetID as i32),)*)*)*
                    }
                }
                
                pub fn state(&self) -> PacketState {
                    match self {
                        $($($(Packet::$name(_) => PacketState::$state,)*)*)*
                    }
                }
                
                pub fn direction(&self) -> PacketDirection {
                    match self {
                        $($($(Packet::$name(_) => PacketDirection::$direction,)*)*)*
                    }
                }
            }
            
            impl McSerialize for Packet {
                fn mc_serialize(&self, serializer: &mut McSerializer) -> SerializingResult<()> {
                    let mut length_serializer = McSerializer::new();
                    match self {
                        $($($(Packet::$name(b) => {b.mc_serialize(&mut length_serializer)?}),*)*)*
                    }
                    
                    let packet_id = self.packet_id();
                    
                    let bytes = packet_id.to_bytes(); // getting the bytes is kind of expensive, so cache it
                    
                    VarInt(length_serializer.output.len() as i32 + bytes.len() as i32).mc_serialize(serializer)?;
                    bytes.mc_serialize(serializer)?;
                    serializer.merge(length_serializer);
                    
            
                    Ok(())
                }
            }
            
            impl StateBasedDeserializer for Packet {
                fn deserialize_state<'a>(deserializer: &'a mut McDeserializer, state: PacketState, packet_direction: PacketDirection) -> SerializingResult<'a, Self> {
                    let length = VarInt::mc_deserialize(deserializer)?;

                    let mut sub = deserializer.sub_deserializer_length(length.0 as usize)?;
                    
                    let packet_id = VarInt::mc_deserialize(&mut sub)?;
                    
                    $(
                        if state == PacketState::$state {
                            $(
                                if packet_direction == PacketDirection::$direction {
                                    match packet_id.0 {
                                        $(
                                            $packetID => {
                                                let a = $name_body::mc_deserialize(&mut sub);

                                                if let Ok(a) = a {
                                                    return Ok(Packet::$name(a));
                                                }
                                            }
                                        )*
                                        
                                            _ => {}
                                    }
                                }
                            )*
                        }
                    )*
                    
                    return Err(SerializingErr::UniqueFailure("Could not find matching type.".to_string()));
                }
            }
        };
    }
    
    #[macro_export]
    macro_rules! pac {
        ($stru: ident => {
            ($state: ident) => {
                $($name: ident, $name_body: ident, $packetID: literal => {
                    $($field: ident: $t: ty),*
                }),* 
            },*
        }) => {
            $(
                $(
                pub struct $name_body { // The body struct of the packet
                    $(pub(crate) $field: $t),*
                }
                )*
            )*
            
            pub enum stru {
                $(
                    $(
                        $name($name_body)
                    )*
                )*
            }
            
            impl stru {
                pub fn here() {
                    
                }
            }
        }
    }

    /// Defines the structs for some fields for packets. This is most frequently used for nested
    /// fields without the use of Optional<T>
    #[macro_export]
    macro_rules! component_struct {
        ($name: ident => {
            $($field: ident: $t: ty),*
        }) => {
            #[derive(Debug, Clone, PartialEq, Eq)]
            pub struct $name { // The body struct of the packet
                $($field: $t),*
            }

            impl McDeserialize for $name {
                fn mc_deserialize<'a>(deserializer: &'a mut McDeserializer) -> SerializingResult<'a, Self> {
                    let s = Self {
                        $($field: <$t>::mc_deserialize(deserializer)?,)*
                    };

                    Ok(s)
                }
            }

            impl McSerialize for $name {
                fn mc_serialize(&self, serializer: &mut McSerializer) -> SerializingResult<()> {
                    $(self.$field.mc_serialize(serializer)?;)*

                    Ok(())
                }
            }
        };
    }
}