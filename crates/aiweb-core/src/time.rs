//! Host monotonic timestamps only comparable within matching boot identities.
use crate::{EngineError,EngineResult};
#[derive(Clone,Debug,PartialEq,Eq)]pub struct MonotonicDeadline {pub boot_id:String,pub tick_ms:u64}
impl MonotonicDeadline {
 pub fn expired(&self,current_boot_id:&str,current_tick_ms:u64)->EngineResult<bool>{if self.boot_id!=current_boot_id{return Err(EngineError::Expired)}Ok(current_tick_ms>=self.tick_ms)}
}
#[cfg(test)]mod tests{use super::*;#[test]fn reboot_not_comparable(){let d=MonotonicDeadline{boot_id:"old".into(),tick_ms:100};assert_eq!(d.expired("new",10),Err(EngineError::Expired));}}
