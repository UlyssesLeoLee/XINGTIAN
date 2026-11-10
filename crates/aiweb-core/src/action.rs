//! Serialized fixture actions, semantic-only snapshots, never automatic redispatch.
use crate::{EngineError,EngineResult};
use crate::state::{PageVersion,Dependency,Precondition};
use std::collections::BTreeMap;
use std::sync::{Arc,Mutex};
#[derive(Clone,Debug,PartialEq,Eq)]
pub struct Element{pub selector:String,pub value:String,pub enabled:bool,pub visible:bool}
#[derive(Clone,Debug)]
pub struct Snapshot{pub version:PageVersion,pub elements:Vec<Element>,pub coverage:&'static str,pub trust_label:&'static str}
#[derive(Debug)]
struct Page {version:PageVersion,elements:BTreeMap<u64,Element>,next_node:u64,dispatched:u64}
#[derive(Clone)]
pub struct FixturePage {inner:Arc<Mutex<Page>>}
impl FixturePage {
 pub fn new()->Self{ Self{inner:Arc::new(Mutex::new(Page{version:PageVersion::default(),elements:BTreeMap::new(),next_node:1,dispatched:0}))}}
 pub fn insert(&self,selector:&str,value:&str)->u64{
   let mut p=self.inner.lock().expect("poisoned fixture mutex");let id=p.next_node;p.next_node+=1;
   p.elements.insert(id,Element{selector:selector.into(),value:value.into(),enabled:true,visible:true});
   p.version.dom_version+=1;p.version.projection_revision+=1;id
 }
 pub fn replace(&self,id:u64,selector:&str,value:&str)->EngineResult<u64>{
   let mut p=self.inner.lock().expect("poisoned fixture mutex");if p.elements.remove(&id).is_none(){return Err(EngineError::InvalidLocator)}
   let next=p.next_node;p.next_node+=1;p.elements.insert(next,Element{selector:selector.into(),value:value.into(),visible:true,enabled:true});p.version.dom_version+=1;p.version.projection_revision+=1;Ok(next)
 }
 pub fn snapshot(&self)->Snapshot{
  let p=self.inner.lock().expect("poisoned fixture mutex");Snapshot{version:p.version.clone(),elements:p.elements.values().cloned().collect(),coverage:"semantic_only_fixture",trust_label:"untrusted_web_content"}
 }
 pub fn set_focus_or_scroll(&self){let mut p=self.inner.lock().expect("poisoned fixture mutex");p.version.user_interaction_revision+=1;}
 pub fn set_geometry_revision(&self){let mut p=self.inner.lock().expect("poisoned fixture mutex");p.version.style_layout_revision+=1;}
 pub fn navigate(&self){let mut p=self.inner.lock().expect("poisoned fixture mutex");p.elements.clear();p.version.document_epoch+=1;p.version.navigation_revision+=1;p.version.projection_revision+=1;}
 /// Strict CAS linearizes validation and mutation under same fixture lock.
 pub fn fill_cas(&self, selector:&str, value:&str, pre:&Precondition)->EngineResult<()> {
  let mut p=self.inner.lock().expect("poisoned fixture mutex");
  pre.validate(&p.version,&[Dependency::FormValue,Dependency::DomTree,Dependency::Policy])?;
  let matches:Vec<u64>=p.elements.iter().filter(|(_,e)|e.selector==selector).map(|(id,_)|*id).collect();
  if matches.len()!=1{return Err(EngineError::InvalidLocator)}
  let e=p.elements.get_mut(&matches[0]).ok_or(EngineError::InvalidLocator)?;
  if !e.enabled||!e.visible{return Err(EngineError::ActionabilityFailed)}
  if e.value!=value {e.value=value.to_string();p.version.form_control_revision+=1;p.version.projection_revision+=1;}
  p.dispatched+=1;Ok(())
 }
 /// One lookup and dispatch barrier per call. No retry on ambiguous outcome.
 pub fn fill_dynamic(&self,selector:&str,value:&str)->EngineResult<()> {
  let mut p=self.inner.lock().expect("poisoned fixture mutex");
  let matches:Vec<u64>=p.elements.iter().filter(|(_,e)|e.selector==selector).map(|(id,_)|*id).collect();
  if matches.len()!=1{return Err(EngineError::InvalidLocator)}
  let e=p.elements.get_mut(&matches[0]).ok_or(EngineError::InvalidLocator)?;
  if !e.enabled||!e.visible{return Err(EngineError::ActionabilityFailed)}
  if e.value!=value {e.value=value.to_string();p.version.form_control_revision+=1;p.version.projection_revision+=1;}
  p.dispatched+=1;Ok(())
 }
 pub fn dispatch_count(&self)->u64{self.inner.lock().expect("poisoned fixture mutex").dispatched}
}
impl Default for FixturePage {fn default()->Self{Self::new()}}
#[cfg(test)] mod tests{
 use super::*;
 #[test]fn old_form_value_cannot_overwrite(){let page=FixturePage::new();page.insert("#name","old");let snap=page.snapshot();let guard=Precondition{expected:snap.version.clone(),read_footprint:vec![Dependency::FormValue,Dependency::DomTree,Dependency::Policy]};page.fill_cas("#name","A",&guard).unwrap();assert_eq!(page.fill_cas("#name","B",&guard),Err(EngineError::StaleSnapshot));assert_eq!(page.snapshot().elements[0].value,"A");}
 #[test]fn stale_navigation_rejected(){let page=FixturePage::new();page.insert("#name","old");let s=page.snapshot();page.navigate();page.insert("#name","new");assert_eq!(page.fill_cas("#name","X",&Precondition{expected:s.version,read_footprint:vec![Dependency::FormValue,Dependency::DomTree,Dependency::Policy]}),Err(EngineError::StaleSnapshot));}
 #[test]fn ambiguous_selector_no_dispatch(){let page=FixturePage::new();page.insert("button","1");page.insert("button","2");assert_eq!(page.fill_dynamic("button","x"),Err(EngineError::InvalidLocator));assert_eq!(page.dispatch_count(),0);}
}
