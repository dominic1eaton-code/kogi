//! PortfolioFederation — multi-system CRDT sync between PortfolioSystem nodes.

use std::collections::HashMap;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::crdt::{CrdtLog, VectorClock};

pub type PeerId = Uuid;

/// Record tracking a known federation peer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FederationPeer {
    pub peer_id: PeerId,
    pub name: String,
    pub endpoint_url: String,
    pub last_seen: DateTime<Utc>,
    pub trusted: bool,
    pub vector_clock: VectorClock,
}

impl FederationPeer {
    pub fn new(name: impl Into<String>, endpoint_url: impl Into<String>, trusted: bool) -> Self {
        Self {
            peer_id: Uuid::new_v4(),
            name: name.into(),
            endpoint_url: endpoint_url.into(),
            last_seen: Utc::now(),
            trusted,
            vector_clock: VectorClock::new(),
        }
    }
}

/// Manages multiple PortfolioSystem instances (one per node or organisation).
/// `sync_crdt(source, target)` pushes the source's CrdtLog into the target.
#[derive(Debug, Default)]
pub struct PortfolioFederation {
    pub peers: HashMap<PeerId, FederationPeer>,
}

impl PortfolioFederation {
    pub fn new() -> Self { Self::default() }

    pub fn register_peer(&mut self, peer: FederationPeer) -> PeerId {
        let id = peer.peer_id;
        self.peers.insert(id, peer);
        id
    }

    pub fn get_peer(&self, id: PeerId) -> Option<&FederationPeer> {
        self.peers.get(&id)
    }

    pub fn mark_seen(&mut self, id: PeerId) {
        if let Some(peer) = self.peers.get_mut(&id) {
            peer.last_seen = Utc::now();
        }
    }

    pub fn stale_peers(&self, threshold_seconds: i64) -> Vec<&FederationPeer> {
        let now = Utc::now();
        self.peers.values()
            .filter(|p| (now - p.last_seen).num_seconds() > threshold_seconds)
            .collect()
    }

    /// Return delta operations for a peer (to push over the wire).
    pub fn delta_for_peer(&self, peer_id: &str, log: &CrdtLog) -> Vec<&crate::crdt::CrdtOperation> {
        log.delta_since(peer_id).iter().collect()
    }
}
