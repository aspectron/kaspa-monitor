use crate::imports::*;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PingRequest {}

impl Serializer for PingRequest {
    fn serialize<W: std::io::Write>(&self, writer: &mut W) -> std::io::Result<()> {
        store!(u16, &1, writer)?;
        Ok(())
    }
}

impl Deserializer for PingRequest {
    fn deserialize<R: std::io::Read>(reader: &mut R) -> std::io::Result<Self> {
        let _version = load!(u16, reader)?;
        Ok(Self {})
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PingResponse {}

impl Serializer for PingResponse {
    fn serialize<W: std::io::Write>(&self, writer: &mut W) -> std::io::Result<()> {
        store!(u16, &1, writer)?;
        Ok(())
    }
}

impl Deserializer for PingResponse {
    fn deserialize<R: std::io::Read>(reader: &mut R) -> std::io::Result<Self> {
        let _version = load!(u16, reader)?;
        Ok(Self {})
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GetStatusRequest {}

impl Serializer for GetStatusRequest {
    fn serialize<W: std::io::Write>(&self, writer: &mut W) -> std::io::Result<()> {
        store!(u16, &1, writer)?;
        Ok(())
    }
}

impl Deserializer for GetStatusRequest {
    fn deserialize<R: std::io::Read>(reader: &mut R) -> std::io::Result<Self> {
        let _version = load!(u16, reader)?;
        Ok(Self {})
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GetStatusResponse {
    pub kaspa_monitor_version: String,
    // pub network_id: NetworkId,
}

impl Serializer for GetStatusResponse {
    fn serialize<W: std::io::Write>(&self, writer: &mut W) -> std::io::Result<()> {
        store!(u16, &1, writer)?;
        store!(String, &self.kaspa_monitor_version, writer)?;
        // store!(NetworkId, &self.network_id, writer)?;
        Ok(())
    }
}

impl Deserializer for GetStatusResponse {
    fn deserialize<R: std::io::Read>(reader: &mut R) -> std::io::Result<Self> {
        let _version = load!(u16, reader)?;
        let sparkled_version = load!(String, reader)?;
        // let network_id = load!(NetworkId, reader)?;
        Ok(Self {
            kaspa_monitor_version: sparkled_version,
            // network_id,
        })
    }
}

// ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
// ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
// ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Update {
    Status { status: Arc<Status> },
    Caps { uid: u64, caps: Arc<Caps> },
}

impl Serializer for Update {
    fn serialize<W: std::io::Write>(&self, writer: &mut W) -> std::io::Result<()> {
        store!(u16, &1, writer)?;
        match self {
            Update::Status { status } => {
                store!(u8, &0, writer)?;
                serialize!(Status, &status, writer)?;
            }
            Update::Caps { uid, caps } => {
                store!(u8, &1, writer)?;
                store!(u64, &uid, writer)?;
                serialize!(Caps, &caps, writer)?;
            }
        }
        Ok(())
    }
}

impl Deserializer for Update {
    fn deserialize<R: std::io::Read>(reader: &mut R) -> std::io::Result<Self> {
        let _version = load!(u16, reader)?;
        let update_type = load!(u8, reader)?;
        match update_type {
            0 => {
                let status = deserialize!(Status, reader)?;
                Ok(Update::Status {
                    status: Arc::new(status),
                })
            }
            1 => {
                let uid = load!(u64, reader)?;
                let caps = deserialize!(Caps, reader)?;
                Ok(Update::Caps {
                    uid,
                    caps: Arc::new(caps),
                })
            }
            _ => Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Invalid update type",
            )),
        }
    }
}
