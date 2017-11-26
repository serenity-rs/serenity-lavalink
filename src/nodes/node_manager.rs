use parking_lot::RwLock;
use std::sync::Arc;
use super::{Node, NodeAudioPlayerManager, NodeConfig, SerenityShardManager};
use ::prelude::*;

#[derive(Clone, Debug, Default)]
pub struct NodeManager {
    pub nodes: Arc<RwLock<Vec<Arc<Node>>>>,
    pub player_manager: NodeAudioPlayerManager,
}

impl NodeManager {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_node(&mut self, config: &NodeConfig, manager: SerenityShardManager)
        -> Result<()> {
        let node = Node::connect(config, manager, Arc::clone(&self.player_manager));

        let mut nodes = self.nodes.write();
        nodes.push(Arc::new(node?));

        Ok(())
    }

    pub fn determine_best_node(&self) -> Option<Arc<Node>> {
        let nodes = self.nodes.read();

        let mut record = i32::max_value();
        let mut best = None;

        for node in nodes.iter() {
            let total = Self::get_penalty(node).unwrap_or(0);

            if total < record {
                best = Some(Arc::clone(node));
                record = total;
            }
        }

        best
    }

    pub fn get_penalty(node: &Arc<Node>) -> Result<i32> {
        let state = node.state.read();

        let stats = match state.stats.clone() {
            Some(stats) => stats,
            None => return Err(Error::StatsNotPresent),
        };

        let cpu = 1.05f64.powf(100f64 * stats.cpu.system_load) * 10f64 - 10f64;

        let (deficit_frame, null_frame) = match stats.frame_stats {
            Some(frame_stats) => {
                (
                    1.03f64.powf(500f64 * (f64::from(frame_stats.deficit) / 3000f64)) * 300f64 - 300f64,
                    (1.03f64.powf(500f64 * (f64::from(frame_stats.nulled) / 3000f64)) * 300f64 - 300f64) * 2f64,
                )
            },
            None => (0f64, 0f64),
        };

        Ok(stats.playing_players + cpu as i32 + deficit_frame as i32 + null_frame as i32)
    }

    pub fn close(self) -> bool {
        let nodes = if let Ok(nodes) = Arc::try_unwrap(self.nodes) {
            nodes
        } else {
            error!("could not Arc::try_unwrap self.nodes on close");

            return false;
        };

        let nodes = nodes.into_inner();

        for node in nodes {
            let node = if let Ok(node) = Arc::try_unwrap(node) {
                node
            } else {
                error!("could not Arc::try_unwrap node");
                continue;
            };

            node.close();
        }

        true
    }
}
