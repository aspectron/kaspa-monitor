use crate::imports::*;

// #[allow(dead_code)]
#[derive(Debug, Serialize, Deserialize)]
pub struct Caps {
    // node version
    pub version: String,
    // node system id
    pub system_id: u64,
    // git hash
    pub git_hash: Option<String>,
    // current memory usage in bytes
    pub total_memory: u64,
    // number of cores
    pub cpu_physical_cores: u64,
    // number of available file descriptors
    pub fd_limit: u64,
    // number of available clients
    pub clients_limit: u64,
}

impl Caps {
    pub fn system_id(&self) -> u64 {
        self.system_id
    }
}

impl Serializer for Caps {
    fn serialize<W: std::io::Write>(&self, writer: &mut W) -> std::io::Result<()> {
        store!(u16, &1, writer)?;
        store!(String, &self.version, writer)?;
        store!(u64, &self.system_id, writer)?;
        store!(Option<String>, &self.git_hash, writer)?;
        store!(u64, &self.total_memory, writer)?;
        store!(u64, &self.cpu_physical_cores, writer)?;
        store!(u64, &self.fd_limit, writer)?;
        store!(u64, &self.clients_limit, writer)?;
        Ok(())
    }
}

impl Deserializer for Caps {
    fn deserialize<R: std::io::Read>(reader: &mut R) -> std::io::Result<Self> {
        let _version = load!(u16, reader)?;
        let version = load!(String, reader)?;
        let system_id = load!(u64, reader)?;
        let git_hash = load!(Option<String>, reader)?;
        let total_memory = load!(u64, reader)?;
        let cpu_physical_cores = load!(u64, reader)?;
        let fd_limit = load!(u64, reader)?;
        let clients_limit = load!(u64, reader)?;

        Ok(Self {
            version,
            system_id,
            git_hash,
            total_memory,
            cpu_physical_cores,
            fd_limit,
            clients_limit,
        })
    }
}
