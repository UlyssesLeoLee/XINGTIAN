//! Tenant-limited, deterministic round-robin with bounded queue and accounting.
//! Fairness/performance targets in DD remain POC-gated; this is NOT distributed DRR.
use crate::{EngineError, EngineResult};
use std::collections::{BTreeMap, VecDeque};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Budget { pub cpu_units: u32, pub memory_units: u32 }
impl Budget {
    fn fits(&self, other: &Budget)->bool{self.cpu_units>=other.cpu_units&&self.memory_units>=other.memory_units}
    fn deduct(&mut self, other:Budget){self.cpu_units-=other.cpu_units;self.memory_units-=other.memory_units}
    fn add(&mut self, other:Budget){self.cpu_units=self.cpu_units.saturating_add(other.cpu_units);self.memory_units=self.memory_units.saturating_add(other.memory_units);}
}
#[derive(Clone, Debug)]
pub struct Job { pub id:String, pub tenant:String, pub cost:Budget }
pub struct Scheduler { queues:BTreeMap<String,VecDeque<Job>>, order:VecDeque<String>, max_queued:usize, queued:usize, remaining:Budget, maximum:Budget }
impl Scheduler {
 pub fn new(max_queued:usize, maximum:Budget)->Self{Self{queues:BTreeMap::new(),order:VecDeque::new(),max_queued,queued:0,remaining:maximum,maximum}}
 pub fn submit(&mut self,job:Job)->EngineResult<()> {
    if job.id.is_empty() || job.tenant.is_empty() || job.cost.cpu_units==0 || job.cost.memory_units==0 {return Err(EngineError::InvalidPrecondition)}
    if self.queued>=self.max_queued||!self.maximum.fits(&job.cost){return Err(EngineError::ResourceExhausted)}
    if !self.queues.contains_key(&job.tenant){ self.order.push_back(job.tenant.clone()); }
    self.queues.entry(job.tenant.clone()).or_default().push_back(job);self.queued+=1;Ok(())
 }
 pub fn next(&mut self)->Option<Job>{
    let attempts=self.order.len();
    for _ in 0..attempts {
      let tenant=self.order.pop_front()?;
      let candidate=self.queues.get(&tenant).and_then(|q|q.front());
      let ready=candidate.map(|j|self.remaining.fits(&j.cost)).unwrap_or(false);
      if ready {
        let q=self.queues.get_mut(&tenant)?;
        let job=q.pop_front()?; self.queued-=1;self.remaining.deduct(job.cost);
        if q.is_empty(){self.queues.remove(&tenant);} else {self.order.push_back(tenant);}
        return Some(job);
      }
      self.order.push_back(tenant);
    }
    None
 }
 pub fn release(&mut self,cost:Budget){self.remaining.add(cost);self.remaining.cpu_units=self.remaining.cpu_units.min(self.maximum.cpu_units);self.remaining.memory_units=self.remaining.memory_units.min(self.maximum.memory_units);}
 pub fn waiting(&self)->usize{self.queued}
 pub fn available(&self)->Budget{self.remaining}
}
#[cfg(test)]mod tests{
 use super::*;
 #[test]fn round_robin_tenants(){let mut s=Scheduler::new(5,Budget{cpu_units:3,memory_units:3});for (i,t) in [(1,"A"),(2,"A"),(3,"B")]{s.submit(Job{id:i.to_string(),tenant:t.to_string(),cost:Budget{cpu_units:1,memory_units:1}}).unwrap();}assert_eq!(s.next().unwrap().tenant,"A");assert_eq!(s.next().unwrap().tenant,"B");assert_eq!(s.next().unwrap().tenant,"A");}
 #[test]fn budget_capacity(){let mut s=Scheduler::new(1,Budget{cpu_units:1,memory_units:1});let mk=|id|Job{id,tenant:"t".into(),cost:Budget{cpu_units:1,memory_units:1}};s.submit(mk("1".into())).unwrap();assert_eq!(s.submit(mk("2".into())),Err(EngineError::ResourceExhausted));let first=s.next().unwrap();assert!(s.next().is_none());s.release(first.cost);assert_eq!(s.available().cpu_units,1);}
}
