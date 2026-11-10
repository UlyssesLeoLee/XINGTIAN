//! Attested identity simulation only. THIS IS NOT an OS security boundary.
//! Broker NEVER opens a socket: denial-by-default controlled fixture tests only.
use crate::{EngineError,EngineResult};
use std::collections::BTreeMap;
#[derive(Clone,Debug)]pub struct Principal {tenant:String,agent:String}
impl Principal {pub fn tenant(&self)->&str{&self.tenant}pub fn agent(&self)->&str{&self.agent}}
/// Principal is created by a trusted test harness; no public construction from request body.
pub struct TrustedHarness;
impl TrustedHarness {pub fn principal(tenant:&str,agent:&str)->Principal{Principal{tenant:tenant.into(),agent:agent.into()}}}
#[derive(Clone,Debug)]pub struct BoundChannel{tenant:String,worker_generation:u64,lease_epoch:u64}
pub struct Supervisor;
impl Supervisor {pub fn attest(&self,principal:&Principal,worker_generation:u64,lease_epoch:u64)->BoundChannel{BoundChannel{tenant:principal.tenant.clone(),worker_generation,lease_epoch}}}
#[derive(Clone,Debug)]pub struct EffectPermit {pub tenant:String,pub worker_generation:u64,pub lease_epoch:u64,pub origin:String,pub method:String,pub nonce:String}
#[derive(Default)]pub struct Broker {current:BTreeMap<String,(u64,u64)>,spent:Vec<String>}
impl Broker {
 pub fn set_active(&mut self,tenant:&str,worker_generation:u64,lease_epoch:u64){self.current.insert(tenant.into(),(worker_generation,lease_epoch));}
 /// Validate a would-be outbound effect. Returns authorization ONLY; never emits network bytes.
 pub fn approve_attempt(&mut self,channel:&BoundChannel,permit:&EffectPermit,origin:&str,method:&str)->EngineResult<()> {
   if channel.tenant!=permit.tenant || channel.worker_generation!=permit.worker_generation ||channel.lease_epoch!=permit.lease_epoch{return Err(EngineError::Unauthorized)}
   if self.current.get(&channel.tenant)!=Some(&(channel.worker_generation,channel.lease_epoch)){return Err(EngineError::StaleLease)}
   if permit.origin!=origin || permit.method!=method ||permit.nonce.is_empty(){return Err(EngineError::Unauthorized)}
   // Nonce is scoped to tenant to reject replay, even after dispatch uncertainty.
   let key=format!("{}:{}",channel.tenant,permit.nonce);
   if self.spent.contains(&key){return Err(EngineError::EffectUncertain)}
   self.spent.push(key);Ok(())
 }
}
#[cfg(test)]mod tests{
 use super::*;
 #[test]fn stale_lease_rejected(){let p=TrustedHarness::principal("a","bot");let s=Supervisor;let ch=s.attest(&p,1,1);let mut b=Broker::default();b.set_active("a",1,2);let permit=EffectPermit{tenant:"a".into(),worker_generation:1,lease_epoch:1,origin:"https://x".into(),method:"POST".into(),nonce:"n".into()};assert_eq!(b.approve_attempt(&ch,&permit,"https://x","POST"),Err(EngineError::StaleLease));}
 #[test]fn permit_reuse_rejected(){let p=TrustedHarness::principal("a","bot");let s=Supervisor;let ch=s.attest(&p,1,1);let mut b=Broker::default();b.set_active("a",1,1);let permit=EffectPermit{tenant:"a".into(),worker_generation:1,lease_epoch:1,origin:"https://x".into(),method:"POST".into(),nonce:"n".into()};b.approve_attempt(&ch,&permit,"https://x","POST").unwrap();assert_eq!(b.approve_attempt(&ch,&permit,"https://x","POST"),Err(EngineError::EffectUncertain));}
 #[test]fn redirected_target_rejected(){let p=TrustedHarness::principal("a","bot");let s=Supervisor;let ch=s.attest(&p,1,1);let mut b=Broker::default();b.set_active("a",1,1);let permit=EffectPermit{tenant:"a".into(),worker_generation:1,lease_epoch:1,origin:"https://x".into(),method:"GET".into(),nonce:"n".into()};assert_eq!(b.approve_attempt(&ch,&permit,"https://evil","GET"),Err(EngineError::Unauthorized));}
}
