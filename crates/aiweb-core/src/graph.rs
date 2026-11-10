//! Deterministic Kahn topological sort. Duplicate IDs, invalid edges and cycles fail closed.
use crate::{EngineError, EngineResult};
use std::collections::{BTreeMap, BTreeSet, VecDeque};

#[derive(Clone, Debug)]
pub struct TaskGraph { pub nodes: Vec<String>, pub edges: Vec<(String, String)> }
impl TaskGraph {
    pub fn topo_order(&self, max_nodes: usize, max_edges: usize) -> EngineResult<Vec<String>> {
        if self.nodes.is_empty() || self.nodes.len() > max_nodes || self.edges.len() > max_edges {
            return Err(EngineError::InvalidDag);
        }
        let unique: BTreeSet<_> = self.nodes.iter().collect();
        if unique.len() != self.nodes.len() || self.nodes.iter().any(|n| n.is_empty()) {
            return Err(EngineError::InvalidDag);
        }
        let mut indegree: BTreeMap<String, usize> = self.nodes.iter().map(|n|(n.clone(),0)).collect();
        let mut successors: BTreeMap<String, Vec<String>> = BTreeMap::new();
        let mut edge_set = BTreeSet::new();
        for (src, dst) in &self.edges {
            if !indegree.contains_key(src) || !indegree.contains_key(dst) || !edge_set.insert((src, dst)) {
                return Err(EngineError::InvalidDag);
            }
            if let Some(n) = indegree.get_mut(dst) { *n += 1; }
            successors.entry(src.clone()).or_default().push(dst.clone());
        }
        let mut ready: VecDeque<String> = indegree.iter().filter(|(_,d)| **d==0).map(|(n,_)|n.clone()).collect();
        let mut result = Vec::new();
        while let Some(node)=ready.pop_front(){
            for target in successors.get(&node).into_iter().flatten() {
                if let Some(d)=indegree.get_mut(target){*d -=1; if *d==0 {ready.push_back(target.clone());}}
            }
            result.push(node);
        }
        if result.len()!=self.nodes.len(){ return Err(EngineError::InvalidDag); }
        Ok(result)
    }
}
#[cfg(test)]
mod tests {
 use super::*;
 #[test] fn topo_success() {let g=TaskGraph{nodes:vec!["A".into(),"B".into(),"C".into()],edges:vec![("A".into(),"B".into()),("B".into(),"C".into())]};assert_eq!(g.topo_order(10,10).unwrap(),vec!["A","B","C"]);}
 #[test] fn cycle_rejected() {let g=TaskGraph{nodes:vec!["A".into(),"B".into()],edges:vec![("A".into(),"B".into()),("B".into(),"A".into())]};assert_eq!(g.topo_order(10,10),Err(EngineError::InvalidDag));}
 #[test] fn unknown_edge_rejected() {let g=TaskGraph{nodes:vec!["A".into()],edges:vec![("A".into(),"Z".into())]};assert!(g.topo_order(10,10).is_err());}
}
