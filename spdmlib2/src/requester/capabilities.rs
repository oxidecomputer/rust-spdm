use crate::msgs::VersionEntry;

// After version negotiation, capabilities are negotiated.
#[derive(Debug)]
pub struct CapabilitiesState {
    version: VersionEntry,
}

impl CapabilitiesState {
    pub fn new(version: VersionEntry) -> CapabilitiesState {
        CapabilitiesState { version }
    }
}
