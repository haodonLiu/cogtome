// ============================================================================
// COGTOME v2.0 Graph Data Structures
// ============================================================================

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet, VecDeque};

// ============================================================================
// Core Types
// ============================================================================

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Graph {
    pub nodes: Vec<Node>,
    pub edges: Vec<Edge>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Edge {
    #[serde(default)]
    pub id: Option<String>,
    pub source: String,
    pub target: String,
    #[serde(default)]
    pub label: Option<String>,
    #[serde(default)]
    pub source_handle: Option<String>,
    #[serde(default)]
    pub target_handle: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Position {
    pub x: f64,
    pub y: f64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RetryConfig {
    pub max: u32,
    #[serde(default)]
    pub backoff: BackoffStrategy,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum BackoffStrategy {
    #[default]
    Exponential,
    Linear,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct OnErrorConfig {
    pub strategy: ErrorStrategy,
    #[serde(default)]
    pub fallback_node: Option<String>,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ErrorStrategy {
    Fail,
    Continue,
    Fallback,
}

// ============================================================================
// Node Enum (tagged via "type" field)
// ============================================================================

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "type")]
pub enum Node {
    #[serde(rename = "start")]
    Start {
        id: String,
        #[serde(default)]
        position: Option<Position>,
    },
    #[serde(rename = "unit")]
    Unit {
        id: String,
        unit: String,
        #[serde(default)]
        input: HashMap<String, String>,
        #[serde(default)]
        timeout_override: Option<u64>,
        #[serde(default)]
        retry: Option<RetryConfig>,
        #[serde(default)]
        on_error: Option<OnErrorConfig>,
        #[serde(default)]
        position: Option<Position>,
    },
    #[serde(rename = "if")]
    If {
        id: String,
        condition: String,
        #[serde(default)]
        position: Option<Position>,
    },
    #[serde(rename = "match")]
    Match {
        id: String,
        on: String,
        #[serde(default)]
        position: Option<Position>,
    },
    #[serde(rename = "foreach")]
    Foreach {
        id: String,
        over: String,
        #[serde(default = "default_as_var")]
        as_var: String,
        #[serde(default)]
        parallel: bool,
        #[serde(default = "default_max_iterations")]
        max_iterations: u32,
        subgraph: Graph,
        #[serde(default)]
        position: Option<Position>,
    },
    #[serde(rename = "fork")]
    Fork {
        id: String,
        #[serde(default)]
        position: Option<Position>,
    },
    #[serde(rename = "join")]
    Join {
        id: String,
        #[serde(default)]
        position: Option<Position>,
    },
    #[serde(rename = "return")]
    Return {
        id: String,
        values: HashMap<String, String>,
        #[serde(default)]
        position: Option<Position>,
    },
    #[serde(rename = "motif")]
    MotifRef {
        id: String,
        motif: String,
        #[serde(default)]
        input: HashMap<String, String>,
        #[serde(default)]
        position: Option<Position>,
    },
}

fn default_as_var() -> String {
    "item".to_string()
}

fn default_max_iterations() -> u32 {
    50
}

// ============================================================================
// Node Helpers
// ============================================================================

impl Node {
    pub fn id(&self) -> &str {
        match self {
            Node::Start { id, .. } => id,
            Node::Unit { id, .. } => id,
            Node::If { id, .. } => id,
            Node::Match { id, .. } => id,
            Node::Foreach { id, .. } => id,
            Node::Fork { id, .. } => id,
            Node::Join { id, .. } => id,
            Node::Return { id, .. } => id,
            Node::MotifRef { id, .. } => id,
        }
    }

    pub fn type_name(&self) -> &str {
        match self {
            Node::Start { .. } => "start",
            Node::Unit { .. } => "unit",
            Node::If { .. } => "if",
            Node::Match { .. } => "match",
            Node::Foreach { .. } => "foreach",
            Node::Fork { .. } => "fork",
            Node::Join { .. } => "join",
            Node::Return { .. } => "return",
            Node::MotifRef { .. } => "motif",
        }
    }

    pub fn position(&self) -> Option<&Position> {
        match self {
            Node::Start { position, .. } => position.as_ref(),
            Node::Unit { position, .. } => position.as_ref(),
            Node::If { position, .. } => position.as_ref(),
            Node::Match { position, .. } => position.as_ref(),
            Node::Foreach { position, .. } => position.as_ref(),
            Node::Fork { position, .. } => position.as_ref(),
            Node::Join { position, .. } => position.as_ref(),
            Node::Return { position, .. } => position.as_ref(),
            Node::MotifRef { position, .. } => position.as_ref(),
        }
    }
}

// ============================================================================
// Graph Validation
// ============================================================================

#[derive(Debug, thiserror::Error)]
pub enum GraphValidationError {
    #[error("No start node found")]
    NoStartNode,
    #[error("Multiple start nodes found: {0:?}")]
    MultipleStartNodes(Vec<String>),
    #[error("No return node found")]
    NoReturnNode,
    #[error("Duplicate node id: {0}")]
    DuplicateNodeId(String),
    #[error("Edge references unknown source: {0}")]
    UnknownSource(String),
    #[error("Edge references unknown target: {0}")]
    UnknownTarget(String),
    #[error("Node '{node_id}' (type {node_type}) has {outgoing} outgoing edges, expected {expected}")]
    WrongOutgoingEdgeCount {
        node_id: String,
        node_type: String,
        outgoing: usize,
        expected: String,
    },
    #[error("If node '{0}' missing true/false labeled edges")]
    IfMissingLabels(String),
    #[error("Match node '{0}' has edges without labels")]
    MatchMissingLabels(String),
    #[error("Cycle detected in graph")]
    CycleDetected,
    #[error("Unreachable node: {0}")]
    UnreachableNode(String),
}

impl<'a> Graph {
    /// Validate graph structure before execution.
    pub fn validate(&self) -> Result<(), GraphValidationError> {
        // 1. Build id index and check for duplicates
        let mut ids = HashSet::new();
        let mut start_nodes = vec![];
        let mut return_count = 0;

        for node in &self.nodes {
            if !ids.insert(node.id().to_string()) {
                return Err(GraphValidationError::DuplicateNodeId(node.id().to_string()));
            }
            match node {
                Node::Start { id, .. } => start_nodes.push(id.clone()),
                Node::Return { .. } => return_count += 1,
                _ => {}
            }
        }

        // 2. Start node checks
        if start_nodes.is_empty() {
            return Err(GraphValidationError::NoStartNode);
        }
        if start_nodes.len() > 1 {
            return Err(GraphValidationError::MultipleStartNodes(start_nodes));
        }
        if return_count == 0 {
            return Err(GraphValidationError::NoReturnNode);
        }

        // 3. Edge references
        for edge in &self.edges {
            if !ids.contains(&edge.source) {
                return Err(GraphValidationError::UnknownSource(edge.source.clone()));
            }
            if !ids.contains(&edge.target) {
                return Err(GraphValidationError::UnknownTarget(edge.target.clone()));
            }
        }

        // 4. Build adjacency and check outgoing edge constraints
        let outgoing: HashMap<String, Vec<&Edge>> = self
            .edges
            .iter()
            .fold(HashMap::new(), |mut acc, e| {
                acc.entry(e.source.clone()).or_default().push(e);
                acc
            });

        for node in &self.nodes {
            let id = node.id();
            let outs = outgoing.get(id).map(|v| v.len()).unwrap_or(0);

            match node {
                Node::Start { .. } => {
                    if outs < 1 {
                        return Err(GraphValidationError::WrongOutgoingEdgeCount {
                            node_id: id.to_string(),
                            node_type: "start".to_string(),
                            outgoing: outs,
                            expected: ">= 1".to_string(),
                        });
                    }
                }
                Node::Unit { .. } | Node::Join { .. } | Node::MotifRef { .. } => {
                    if outs != 1 {
                        return Err(GraphValidationError::WrongOutgoingEdgeCount {
                            node_id: id.to_string(),
                            node_type: node.type_name().to_string(),
                            outgoing: outs,
                            expected: "1".to_string(),
                        });
                    }
                }
                Node::If { .. } => {
                    if outs != 2 {
                        return Err(GraphValidationError::WrongOutgoingEdgeCount {
                            node_id: id.to_string(),
                            node_type: "if".to_string(),
                            outgoing: outs,
                            expected: "2".to_string(),
                        });
                    }
                    let edges = outgoing.get(id).unwrap();
                    let has_true = edges.iter().any(|e| e.label.as_deref() == Some("true"));
                    let has_false = edges.iter().any(|e| e.label.as_deref() == Some("false"));
                    if !has_true || !has_false {
                        return Err(GraphValidationError::IfMissingLabels(id.to_string()));
                    }
                }
                Node::Match { .. } => {
                    if outs < 1 {
                        return Err(GraphValidationError::WrongOutgoingEdgeCount {
                            node_id: id.to_string(),
                            node_type: "match".to_string(),
                            outgoing: outs,
                            expected: ">= 1".to_string(),
                        });
                    }
                    if let Some(edges) = outgoing.get(id) {
                        if edges.iter().any(|e| e.label.is_none()) {
                            return Err(GraphValidationError::MatchMissingLabels(id.to_string()));
                        }
                    }
                }
                Node::Fork { .. } => {
                    if outs < 1 {
                        return Err(GraphValidationError::WrongOutgoingEdgeCount {
                            node_id: id.to_string(),
                            node_type: "fork".to_string(),
                            outgoing: outs,
                            expected: ">= 1".to_string(),
                        });
                    }
                }
                Node::Foreach { subgraph, .. } => {
                    if outs != 1 {
                        return Err(GraphValidationError::WrongOutgoingEdgeCount {
                            node_id: id.to_string(),
                            node_type: "foreach".to_string(),
                            outgoing: outs,
                            expected: "1".to_string(),
                        });
                    }
                    // Recursively validate subgraph
                    subgraph.validate()?;
                }
                Node::Return { .. } => {
                    if outs != 0 {
                        return Err(GraphValidationError::WrongOutgoingEdgeCount {
                            node_id: id.to_string(),
                            node_type: "return".to_string(),
                            outgoing: outs,
                            expected: "0".to_string(),
                        });
                    }
                }
            }
        }

        // 5. Cycle detection (DFS)
        if self.has_cycle() {
            return Err(GraphValidationError::CycleDetected);
        }

        // 6. Reachability (all nodes reachable from start)
        let reachable = self.reachable_from(&start_nodes[0]);
        for node in &self.nodes {
            if !reachable.contains(node.id()) {
                return Err(GraphValidationError::UnreachableNode(node.id().to_string()));
            }
        }

        Ok(())
    }

    fn has_cycle(&self) -> bool {
        let mut adj: HashMap<&str, Vec<&str>> = HashMap::new();
        for edge in &self.edges {
            adj.entry(&edge.source).or_default().push(&edge.target);
        }

        let mut visited = HashSet::new();
        let mut rec_stack = HashSet::new();

        for node in &self.nodes {
            if Self::dfs_cycle(node.id(), &adj, &mut visited, &mut rec_stack) {
                return true;
            }
        }
        false
    }

    fn dfs_cycle(
        node: &str,
        adj: &HashMap<&str, Vec<&str>>,
        visited: &mut HashSet<String>,
        rec_stack: &mut HashSet<String>,
    ) -> bool {
        let node_str = node.to_string();
        if rec_stack.contains(&node_str) {
            return true;
        }
        if visited.contains(&node_str) {
            return false;
        }

        visited.insert(node_str.clone());
        rec_stack.insert(node_str.clone());

        if let Some(neighbors) = adj.get(node) {
            for &neighbor in neighbors {
                if Self::dfs_cycle(neighbor, adj, visited, rec_stack) {
                    return true;
                }
            }
        }

        rec_stack.remove(&node_str);
        false
    }

    fn reachable_from(&self, start: &str) -> HashSet<String> {
        let mut adj: HashMap<&str, Vec<&str>> = HashMap::new();
        for edge in &self.edges {
            adj.entry(&edge.source).or_default().push(&edge.target);
        }

        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();
        queue.push_back(start);
        visited.insert(start.to_string());

        while let Some(current) = queue.pop_front() {
            if let Some(neighbors) = adj.get(current) {
                for &neighbor in neighbors {
                    if visited.insert(neighbor.to_string()) {
                        queue.push_back(neighbor);
                    }
                }
            }
        }

        visited
    }

    /// Find a node by id
    pub fn find_node(&self, id: &str) -> Option<&Node> {
        self.nodes.iter().find(|n| n.id() == id)
    }

    /// Get outgoing edges from a node
    pub fn outgoing_edges(&self, node_id: &str) -> Vec<&Edge> {
        self.edges.iter().filter(|e| e.source == node_id).collect()
    }

    /// Get incoming edges to a node
    pub fn incoming_edges(&self, node_id: &str) -> Vec<&Edge> {
        self.edges.iter().filter(|e| e.target == node_id).collect()
    }

    /// Topological sort for execution order
    pub fn topological_sort(&self) -> Vec<&Node> {
        let mut in_degree: HashMap<&str, usize> = HashMap::new();
        let mut adj: HashMap<&str, Vec<&str>> = HashMap::new();

        for node in &self.nodes {
            in_degree.entry(node.id()).or_insert(0);
            adj.entry(node.id()).or_default();
        }

        for edge in &self.edges {
            *in_degree.entry(&edge.target).or_insert(0) += 1;
            adj.entry(&edge.source).or_default().push(&edge.target);
        }

        let mut queue: Vec<&str> = in_degree
            .iter()
            .filter(|(_, &d)| d == 0)
            .map(|(&k, _)| k)
            .collect();

        let mut result: Vec<&Node> = Vec::new();
        while let Some(current) = queue.pop() {
            if let Some(node) = self.nodes.iter().find(|n| n.id() == current) {
                result.push(node);
            }
            if let Some(neighbors) = adj.get(current) {
                for &neighbor in neighbors {
                    if let Some(deg) = in_degree.get_mut(neighbor) {
                        *deg -= 1;
                        if *deg == 0 {
                            queue.push(neighbor);
                        }
                    }
                }
            }
        }

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_simple_graph() {
        let graph = Graph {
            nodes: vec![
                Node::Start { id: "start".into(), position: None },
                Node::Unit {
                    id: "fetch".into(),
                    unit: "http-get".into(),
                    input: HashMap::new(),
                    position: None,
                    timeout_override: None,
                    retry: None,
                    on_error: None,
                },
                Node::Return {
                    id: "done".into(),
                    values: HashMap::new(),
                    position: None,
                },
            ],
            edges: vec![
                Edge { id: None, source: "start".into(), target: "fetch".into(), label: None, source_handle: None, target_handle: None },
                Edge { id: None, source: "fetch".into(), target: "done".into(), label: None, source_handle: None, target_handle: None },
            ],
        };

        assert!(graph.validate().is_ok());
    }

    #[test]
    fn test_missing_start() {
        let graph = Graph {
            nodes: vec![
                Node::Unit {
                    id: "fetch".into(),
                    unit: "http-get".into(),
                    input: HashMap::new(),
                    position: None,
                    timeout_override: None,
                    retry: None,
                    on_error: None,
                },
            ],
            edges: vec![],
        };

        assert!(matches!(graph.validate(), Err(GraphValidationError::NoStartNode)));
    }

    #[test]
    fn test_cycle_detection() {
        // Graph with a cycle: start -> a -> b -> a (cycle back to a)
        // Has return node so it passes the "has return" check before cycle detection
        let graph = Graph {
            nodes: vec![
                Node::Start { id: "start".into(), position: None },
                Node::Unit {
                    id: "a".into(),
                    unit: "x".into(),
                    input: HashMap::new(),
                    position: None,
                    timeout_override: None,
                    retry: None,
                    on_error: None,
                },
                Node::Unit {
                    id: "b".into(),
                    unit: "y".into(),
                    input: HashMap::new(),
                    position: None,
                    timeout_override: None,
                    retry: None,
                    on_error: None,
                },
                Node::Return {
                    id: "done".into(),
                    values: HashMap::new(),
                    position: None,
                },
            ],
            edges: vec![
                Edge { id: None, source: "start".into(), target: "a".into(), label: None, source_handle: None, target_handle: None },
                Edge { id: None, source: "a".into(), target: "b".into(), label: None, source_handle: None, target_handle: None },
                Edge { id: None, source: "b".into(), target: "a".into(), label: None, source_handle: None, target_handle: None },
            ],
        };

        assert!(matches!(graph.validate(), Err(GraphValidationError::CycleDetected)));
    }

    #[test]
    fn test_if_node_requires_true_false_edges() {
        // Graph where if node has 2 outgoing edges but both are labeled "true" (missing "false")
        let graph = Graph {
            nodes: vec![
                Node::Start { id: "start".into(), position: None },
                Node::If { id: "gate".into(), condition: "${x}".into(), position: None },
                Node::Return {
                    id: "done".into(),
                    values: HashMap::new(),
                    position: None,
                },
                Node::Return {
                    id: "done2".into(),
                    values: HashMap::new(),
                    position: None,
                },
            ],
            edges: vec![
                Edge { id: None, source: "start".into(), target: "gate".into(), label: None, source_handle: None, target_handle: None },
                Edge { id: None, source: "gate".into(), target: "done".into(), label: Some("true".into()), source_handle: None, target_handle: None },
                Edge { id: None, source: "gate".into(), target: "done2".into(), label: Some("true".into()), source_handle: None, target_handle: None },
            ],
        };

        assert!(matches!(graph.validate(), Err(GraphValidationError::IfMissingLabels(_))));
    }

    #[test]
    fn test_foreach_with_subgraph() {
        let subgraph = Graph {
            nodes: vec![
                Node::Start { id: "__entry__".into(), position: None },
                Node::Unit {
                    id: "proc".into(),
                    unit: "process".into(),
                    input: HashMap::new(),
                    position: None,
                    timeout_override: None,
                    retry: None,
                    on_error: None,
                },
                Node::Return { id: "ret".into(), values: HashMap::new(), position: None },
            ],
            edges: vec![
                Edge { id: None, source: "__entry__".into(), target: "proc".into(), label: None, source_handle: None, target_handle: None },
                Edge { id: None, source: "proc".into(), target: "ret".into(), label: None, source_handle: None, target_handle: None },
            ],
        };

        let graph = Graph {
            nodes: vec![
                Node::Start { id: "start".into(), position: None },
                Node::Foreach {
                    id: "loop".into(),
                    over: "${items}".into(),
                    as_var: "item".into(),
                    parallel: false,
                    max_iterations: 50,
                    subgraph,
                    position: None,
                },
                Node::Return { id: "done".into(), values: HashMap::new(), position: None },
            ],
            edges: vec![
                Edge { id: None, source: "start".into(), target: "loop".into(), label: None, source_handle: None, target_handle: None },
                Edge { id: None, source: "loop".into(), target: "done".into(), label: None, source_handle: None, target_handle: None },
            ],
        };

        assert!(graph.validate().is_ok());
    }
}