//! A narrow version-vector model for fixture actions; not a complete Web Agent Event Loop.
use crate::{EngineError,EngineResult};
#[derive(Clone,Debug,PartialEq,Eq)]
pub struct PageVersion {
 pub context_generation:u64,pub page_generation:u64,pub document_epoch:u64,
 pub navigation_revision:u64,pub dom_version:u64,pub form_control_revision:u64,
 pub user_interaction_revision:u64,pub style_layout_revision:u64,
 pub frame_topology_revision:u64,pub policy_revision:u64,pub projection_revision:u64,
}
impl Default for PageVersion {fn default()->Self {Self{context_generation:1,page_generation:1,document_epoch:1,navigation_revision:1,dom_version:1,form_control_revision:1,user_interaction_revision:1,style_layout_revision:1,frame_topology_revision:1,policy_revision:1,projection_revision:1}}}
#[derive(Clone,Copy,Debug,Eq,PartialEq,Hash)]
pub enum Dependency {DocumentIdentity,DomTree,FormValue,UserInteraction,Geometry,FrameTopology,Policy,Projection}
#[derive(Clone,Debug)]
pub struct Precondition{pub expected:PageVersion,pub read_footprint:Vec<Dependency>}
impl Precondition {
 pub fn validate(&self,actual:&PageVersion,required:&[Dependency])->EngineResult<()> {
     if self.expected.context_generation!=actual.context_generation || self.expected.page_generation!=actual.page_generation ||self.expected.document_epoch!=actual.document_epoch {return Err(EngineError::StaleSnapshot)}
     for dep in required {
       if !self.read_footprint.contains(dep){return Err(EngineError::InvalidPrecondition)}
     }
     for dep in &self.read_footprint {
       let equal=match dep {
         Dependency::DocumentIdentity=>true,
         Dependency::DomTree=>self.expected.dom_version==actual.dom_version,
         Dependency::FormValue=>self.expected.form_control_revision==actual.form_control_revision,
         Dependency::UserInteraction=>self.expected.user_interaction_revision==actual.user_interaction_revision,
         Dependency::Geometry=>self.expected.style_layout_revision==actual.style_layout_revision,
         Dependency::FrameTopology=>self.expected.frame_topology_revision==actual.frame_topology_revision,
         Dependency::Policy=>self.expected.policy_revision==actual.policy_revision,
         Dependency::Projection=>self.expected.projection_revision==actual.projection_revision,
       };
       if !equal{return Err(EngineError::StaleSnapshot)}
     }
     Ok(())
 }
}
#[cfg(test)]mod tests{
 use super::*;
 #[test]fn form_revision_is_not_dom_revision(){let before=PageVersion::default();let mut now=before.clone();now.form_control_revision+=1;assert_eq!(before.dom_version,now.dom_version);assert_eq!(Precondition{expected:before,read_footprint:vec![Dependency::FormValue]}.validate(&now,&[Dependency::FormValue]),Err(EngineError::StaleSnapshot));}
 #[test]fn missing_footprint_fails(){let v=PageVersion::default();assert_eq!(Precondition{expected:v.clone(),read_footprint:vec![Dependency::DomTree]}.validate(&v,&[Dependency::FormValue]),Err(EngineError::InvalidPrecondition));}
}
