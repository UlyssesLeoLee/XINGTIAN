//! Per-stream cursor, bounded retained events, explicit gap. No cross-stream total order.
use crate::{EngineError,EngineResult};
use std::collections::VecDeque;
#[derive(Clone,Debug,PartialEq,Eq)] pub struct Event {pub seq:u64,pub payload:String}
pub struct EventStream {tenant:String,stream_id:String,next_seq:u64,retained:VecDeque<Event>,cap:usize}
impl EventStream {
 pub fn new(tenant:&str,stream_id:&str,cap:usize)->Self{Self{tenant:tenant.into(),stream_id:stream_id.into(),next_seq:1,retained:VecDeque::new(),cap}}
 pub fn push(&mut self,message:&str)->EngineResult<u64>{if self.cap==0{return Err(EngineError::ResourceExhausted)}let seq=self.next_seq;self.next_seq+=1;self.retained.push_back(Event{seq,payload:message.into()});if self.retained.len()>self.cap{self.retained.pop_front();}Ok(seq)}
 pub fn resume(&self,tenant:&str,stream_id:&str,after_seq:u64)->EngineResult<Vec<Event>>{
   if tenant!=self.tenant||stream_id!=self.stream_id{return Err(EngineError::Unauthorized)}
   let earliest=self.retained.front().map(|e|e.seq).unwrap_or(self.next_seq);
   if after_seq.saturating_add(1)<earliest{return Err(EngineError::EventGap)}
   if after_seq>=self.next_seq{return Err(EngineError::InvalidPrecondition)}
   Ok(self.retained.iter().filter(|e|e.seq>after_seq).cloned().collect())
 }
 pub fn watermark(&self)->u64{self.next_seq-1}
}
#[cfg(test)]mod tests{
 use super::*;
 #[test]fn gap_is_explicit(){let mut s=EventStream::new("t","page",2);s.push("a").unwrap();s.push("b").unwrap();s.push("c").unwrap();assert_eq!(s.resume("t","page",0),Err(EngineError::EventGap));assert_eq!(s.resume("other","page",2),Err(EngineError::Unauthorized));assert_eq!(s.resume("t","page",1).unwrap().len(),2);}
}
