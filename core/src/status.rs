use crate::imports::*;

#[derive(Debug, Serialize, Deserialize)]
pub enum Status {
    Kaspa(Box<KaspaNodeStatus>),
    Sparkle(Box<SparkleNodeStatus>),
}

impl Status {
    pub fn uid(&self) -> u64 {
        match self {
            Status::Kaspa(status) => status.uid,
            Status::Sparkle(status) => status.uid,
        }
    }

    pub fn sid(&self) -> u64 {
        match self {
            Status::Kaspa(status) => status.sid,
            Status::Sparkle(status) => status.sid,
        }
    }

    pub fn name(&self) -> String {
        match self {
            Status::Kaspa(_) => "Kaspa",
            Status::Sparkle(_) => "Sparkle",
        }
        .to_string()
    }
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
    pub metrics_snapshot: MetricsSnapshot,

    pub network_id: NetworkId,
    pub block_count: u64,
    pub header_count: u64,
    pub tip_hashes: Vec<Hash>,
    pub difficulty: f64,
    pub past_median_time: u64,
    pub virtual_parent_hashes: Vec<Hash>,
    pub pruning_point_hash: Hash,
    pub virtual_daa_score: u64,
    pub sink: Hash,
}

impl Serializer for KaspaNodeStatus {
    fn serialize<W: std::io::Write>(&self, writer: &mut W) -> std::io::Result<()> {
        store!(u16, &1, writer)?;
        store!(bool, &self.is_synced, writer)?;
        store!(u64, &self.sid, writer)?;
        store!(u64, &self.uid, writer)?;
        store!(MetricsSnapshot, &self.metrics_snapshot, writer)?;
        store!(NetworkId, &self.network_id, writer)?;
        store!(u64, &self.block_count, writer)?;
        store!(u64, &self.header_count, writer)?;
        store!(Vec<Hash>, &self.tip_hashes, writer)?;
        store!(f64, &self.difficulty, writer)?;
        store!(u64, &self.past_median_time, writer)?;
        store!(Vec<Hash>, &self.virtual_parent_hashes, writer)?;
        store!(Hash, &self.pruning_point_hash, writer)?;
        store!(u64, &self.virtual_daa_score, writer)?;
        store!(Hash, &self.sink, writer)?;
        Ok(())
    }
}

impl Deserializer for KaspaNodeStatus {
    fn deserialize<R: std::io::Read>(reader: &mut R) -> std::io::Result<Self> {
        let _version = load!(u16, reader)?;
        let is_synced = load!(bool, reader)?;
        let sid = load!(u64, reader)?;
        let uid = load!(u64, reader)?;
        let metrics_snapshot = load!(MetricsSnapshot, reader)?;
        let network_id = load!(NetworkId, reader)?;
        let block_count = load!(u64, reader)?;
        let header_count = load!(u64, reader)?;
        let tip_hashes = load!(Vec<Hash>, reader)?;
        let difficulty = load!(f64, reader)?;
        let past_median_time = load!(u64, reader)?;
        let virtual_parent_hashes = load!(Vec<Hash>, reader)?;
        let pruning_point_hash = load!(Hash, reader)?;
        let virtual_daa_score = load!(u64, reader)?;
        let sink = load!(Hash, reader)?;

        Ok(Self {
            sid,
            uid,
            is_synced,
            metrics_snapshot,
            network_id,
            block_count,
            header_count,
            tip_hashes,
            difficulty,
            past_median_time,
            virtual_parent_hashes,
            pruning_point_hash,
            virtual_daa_score,
            sink,
        })
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SparkleNodeStatus {
    pub sid: u64,
    pub uid: u64,
}

impl Serializer for SparkleNodeStatus {
    fn serialize<W: std::io::Write>(&self, writer: &mut W) -> std::io::Result<()> {
        store!(u16, &1, writer)?;
        store!(u64, &self.sid, writer)?;
        store!(u64, &self.uid, writer)?;
        Ok(())
    }
}

impl Deserializer for SparkleNodeStatus {
    fn deserialize<R: std::io::Read>(reader: &mut R) -> std::io::Result<Self> {
        let _version = load!(u16, reader)?;
        let sid = load!(u64, reader)?;
        let uid = load!(u64, reader)?;
        Ok(Self { sid, uid })
    }
}
