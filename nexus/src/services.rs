use crate::imports::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ServiceKind {
    Kaspa,
    Sparkle,
}

impl Display for ServiceKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            ServiceKind::Kaspa => "kaspa",
            ServiceKind::Sparkle => "sparkle",
        };
        f.write_str(s)
    }
}
