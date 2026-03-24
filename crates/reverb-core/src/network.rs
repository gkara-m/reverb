pub enum PacketType {
    Action,
    Query,
}

pub struct Packet {
    version: String,
    username: String,
    usergroup: String,
    packet_type: PacketType,
    payload: Vec<u8>,
}

impl Packet {
    pub fn new(
        version: String,
        username: String,
        usergroup: String,
        packet_type: PacketType,
        payload: Vec<u8>,
    ) -> Result<Self, String> {
        if username.len() > 32 {
            return Err("PLACEHOLDER Error: username too long".to_string());
        }
        if usergroup.len() > 16 {
            return Err("PLACEHOLDER Error: group name too long".to_string());
        }
        if payload.len() > 1024 {
            return Err("PLACEHOLDER Error: group name too long".to_string());
        }

        Ok(Packet {
            version,
            username,
            usergroup,
            packet_type,
            payload,
        })
    }

    pub fn version(&self) -> &str {
        &self.version
    }
    pub fn username(&self) -> &str {
        &self.username
    }
    pub fn usergroup(&self) -> &str {
        &self.usergroup
    }
    pub fn packet_type(&self) -> &PacketType {
        &self.packet_type
    }
    pub fn payload(&self) -> &Vec<u8> {
        &self.payload
    }
}
