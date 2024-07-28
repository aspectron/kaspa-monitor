use crate::imports::*;

#[derive(Debug, Serialize, Deserialize)]
pub enum Status {
    Kaspa(Box<KaspaNodeStatus>),
    Sparkle(Box<SparkleNodeStatus>),
}

impl Serializer for Status {
    fn serialize<W: std::io::Write>(&self, writer: &mut W) -> std::io::Result<()> {
        store!(u16, &1, writer)?;
        match self {
            Status::Kaspa(status) => {
                store!(u8, &1, writer)?;
                serialize!(KaspaNodeStatus, &status, writer)?;
            }
            Status::Sparkle(status) => {
                store!(u8, &1, writer)?;
                serialize!(SparkleNodeStatus, &status, writer)?;
            }
        }
        Ok(())
    }
}

impl Deserializer for Status {
    fn deserialize<R: std::io::Read>(reader: &mut R) -> std::io::Result<Self> {
        let _version = load!(u16, reader)?;
        let kind = load!(u8, reader)?;
        match kind {
            1 => {
                let status = deserialize!(KaspaNodeStatus, reader)?;
                Ok(Status::Kaspa(Box::new(status)))
            }
            2 => {
                let status = deserialize!(SparkleNodeStatus, reader)?;
                Ok(Status::Sparkle(Box::new(status)))
            }
            _ => Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Invalid kind while deserializing `Status`",
            )),
        }
    }
}

impl From<KaspaNodeStatus> for Status {
    fn from(status: KaspaNodeStatus) -> Self {
        Status::Kaspa(Box::new(status))
    }
}

impl From<SparkleNodeStatus> for Status {
    fn from(status: SparkleNodeStatus) -> Self {
        Status::Sparkle(Box::new(status))
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct KaspaNodeStatus {
    pub sid: u64,
    pub uid: u64,
    pub is_synced: bool,
    pub block_dag_info: GetBlockDagInfoResponse,
    pub metrics_snapshot: MetricsSnapshot,
}

impl Serializer for KaspaNodeStatus {
    fn serialize<W: std::io::Write>(&self, writer: &mut W) -> std::io::Result<()> {
        store!(u16, &1, writer)?;
        store!(bool, &self.is_synced, writer)?;
        store!(u64, &self.sid, writer)?;
        store!(u64, &self.uid, writer)?;
        serialize!(GetBlockDagInfoResponse, &self.block_dag_info, writer)?;
        store!(MetricsSnapshot, &self.metrics_snapshot, writer)?;
        Ok(())
    }
}

impl Deserializer for KaspaNodeStatus {
    fn deserialize<R: std::io::Read>(reader: &mut R) -> std::io::Result<Self> {
        let _version = load!(u16, reader)?;
        let is_synced = load!(bool, reader)?;
        let sid = load!(u64, reader)?;
        let uid = load!(u64, reader)?;
        let block_dag_info = deserialize!(GetBlockDagInfoResponse, reader)?;
        let metrics_snapshot = load!(MetricsSnapshot, reader)?;
        Ok(Self {
            sid,
            uid,
            is_synced,
            block_dag_info,
            metrics_snapshot,
        })
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SparkleNodeStatus {}

impl Serializer for SparkleNodeStatus {
    fn serialize<W: std::io::Write>(&self, writer: &mut W) -> std::io::Result<()> {
        store!(u16, &1, writer)?;
        Ok(())
    }
}

impl Deserializer for SparkleNodeStatus {
    fn deserialize<R: std::io::Read>(reader: &mut R) -> std::io::Result<Self> {
        let _version = load!(u16, reader)?;
        Ok(Self {})
    }
}
