//! In-memory effect accounting for deterministic tests only. No database/WAL durability.
//! HTTP response is transport evidence, NOT business success. Uncertain may not auto replay.
use crate::{EngineError,EngineResult};
use std::collections::BTreeMap;
#[derive(Clone,Debug,Eq,PartialEq)]
pub enum EffectState {Accepted,Executing,ConfirmedTransport,FailedNoEffect,Uncertain,Rejected}
#[derive(Clone,Debug,Eq,PartialEq)]
pub enum BusinessOutcome {Unverified,ConfirmedByTrustedReconciliation}
#[derive(Clone,Debug)]
pub struct Effect {pub id:String,pub action_id:String,pub state:EffectState,pub business:BusinessOutcome,pub attempts:u32}
#[derive(Default)]
pub struct EffectLedger {effects:BTreeMap<String,Effect>,idempotency:BTreeMap<(String,String,String),(String,String)>}
impl EffectLedger {
 /// Tenant + request kind + idempotency key -> same action; different digest -> reject.
 pub fn reserve_action(&mut self,tenant:&str,kind:&str,key:&str,digest:&str,action_id:&str)->EngineResult<String>{
   if tenant.is_empty()||kind.is_empty()||key.is_empty()||digest.is_empty(){return Err(EngineError::InvalidPrecondition)}
   let composite=(tenant.into(),kind.into(),key.into());
   if let Some((old_digest,old_id))=self.idempotency.get(&composite){return if old_digest==digest{Ok(old_id.clone())}else{Err(EngineError::IdempotencyKeyReuse)}}
   self.idempotency.insert(composite,(digest.into(),action_id.into()));Ok(action_id.into())
 }
 pub fn record_intent(&mut self,id:&str,action_id:&str)->EngineResult<()> {
  if self.effects.contains_key(id){return Err(EngineError::IdempotencyKeyReuse)}
  self.effects.insert(id.into(),Effect{id:id.into(),action_id:action_id.into(),state:EffectState::Accepted,business:BusinessOutcome::Unverified,attempts:0});Ok(())
 }
 pub fn dispatch_once(&mut self,id:&str)->EngineResult<()> {
   let e=self.effects.get_mut(id).ok_or(EngineError::InvalidPrecondition)?;
   if e.state!=EffectState::Accepted {return Err(EngineError::EffectUncertain)}
   e.state=EffectState::Executing;e.attempts+=1;Ok(())
 }
 pub fn mark_disconnect(&mut self,id:&str)->EngineResult<()> {
   let e=self.effects.get_mut(id).ok_or(EngineError::InvalidPrecondition)?;
   if e.state!=EffectState::Executing{return Err(EngineError::InvalidPrecondition)}
   e.state=EffectState::Uncertain;Ok(())
 }
 pub fn observe_http_response(&mut self,id:&str)->EngineResult<()> {
   let e=self.effects.get_mut(id).ok_or(EngineError::InvalidPrecondition)?;
   if e.state!=EffectState::Executing{return Err(EngineError::InvalidPrecondition)}
   e.state=EffectState::ConfirmedTransport;Ok(())
 }
 pub fn reconcile_business(&mut self,id:&str,proof_from_trusted_service:bool)->EngineResult<()> {
   if !proof_from_trusted_service{return Err(EngineError::Unauthorized)}
   let e=self.effects.get_mut(id).ok_or(EngineError::InvalidPrecondition)?;
   if e.state!=EffectState::ConfirmedTransport && e.state!=EffectState::Uncertain {return Err(EngineError::InvalidPrecondition)}
   e.business=BusinessOutcome::ConfirmedByTrustedReconciliation;Ok(())
 }
 pub fn get(&self,id:&str)->Option<&Effect>{self.effects.get(id)}
}
#[cfg(test)]mod tests{
 use super::*;
 #[test]fn disconnect_no_replay(){let mut l=EffectLedger::default();l.record_intent("e","a").unwrap();l.dispatch_once("e").unwrap();l.mark_disconnect("e").unwrap();assert_eq!(l.dispatch_once("e"),Err(EngineError::EffectUncertain));assert_eq!(l.get("e").unwrap().attempts,1);}
 #[test]fn idempotency_scope(){let mut l=EffectLedger::default();assert_eq!(l.reserve_action("t","fill","k","hash","a").unwrap(),"a");assert_eq!(l.reserve_action("t","fill","k","hash","another").unwrap(),"a");assert_eq!(l.reserve_action("t","fill","k","changed","a"),Err(EngineError::IdempotencyKeyReuse));assert!(l.reserve_action("other","fill","k","changed","a").is_ok());}
 #[test]fn transport_not_business(){let mut l=EffectLedger::default();l.record_intent("e","a").unwrap();l.dispatch_once("e").unwrap();l.observe_http_response("e").unwrap();assert_eq!(l.get("e").unwrap().business,BusinessOutcome::Unverified);}
}
