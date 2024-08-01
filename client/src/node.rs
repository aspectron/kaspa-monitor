use pad::Alignment;

use crate::imports::*;

pub struct Machine {
    caps: Arc<Caps>,
}

impl Machine {
    pub fn new(caps: Arc<Caps>) -> Self {
        Self { caps }
    }

    pub fn caps(&self) -> &Arc<Caps> {
        &self.caps
    }

    pub fn set_caps(&mut self, caps: Arc<Caps>) {
        self.caps = caps;
    }

    pub fn get_caption(&self) -> String {
        format!("{:016x}", self.caps.system_id())
    }
}

pub struct Node {
    pub status: Arc<Status>,
}

impl Node {
    pub fn new(status: Arc<Status>) -> Self {
        Self { status }
    }

    pub fn uid(&self) -> u64 {
        self.status.uid()
    }

    pub fn network_id(&self) -> NetworkId {
        match &*self.status {
            Status::Kaspa(status) => status.network_id,
            Status::Sparkle(_status) => NetworkType::Mainnet.try_into().unwrap(),
        }
    }

    pub fn status(&self) -> &Status {
        &self.status
    }

    pub fn set_status(&mut self, status: Arc<Status>) {
        self.status = status;
    }

    pub fn get_caption(&self) -> String {
        format!(
            "{:016x} {} - {}",
            self.status.uid(),
            self.status.name(),
            self.summary()
        )
    }

    pub fn summary(&self) -> String {
        match &*self.status {
            Status::Kaspa(status) => {
                format!(
                    "{} DAA: {}",
                    status.network_id.to_string().pad_to_width(12),
                    status
                        .virtual_daa_score
                        .separated_string()
                        .pad_to_width_with_alignment(12, Alignment::Right)
                )
            }
            Status::Sparkle(_status) => "".to_string(),
        }
    }
}
