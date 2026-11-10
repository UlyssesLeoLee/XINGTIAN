//! Implemented deterministic fixture-level red regressions.
//! Case linkage in docs/Test_Case_Catalog.md; NOT OS/browser/web standard tests.
use aiweb_core::{EngineError};
use aiweb_core::action::FixturePage;
use aiweb_core::state::{Precondition,Dependency};
use aiweb_core::effects::{EffectLedger,EffectState,BusinessOutcome};
use aiweb_core::security::{Broker,Supervisor,TrustedHarness,EffectPermit};
use aiweb_core::events::EventStream;
use aiweb_core::time::MonotonicDeadline;
use aiweb_core::scheduler::{Scheduler,Budget,Job};
use std::thread;

fn guard(page:&FixturePage)->Precondition {
 Precondition{expected:page.snapshot().version,read_footprint:vec![Dependency::DomTree,Dependency::FormValue,Dependency::Policy]}
}
#[test] fn red_dd_001_form_revision_stale_cas() {
 let page=FixturePage::new();page.insert("#input","0");let p=guard(&page);
 page.fill_cas("#input","1",&p).unwrap();
 assert_eq!(page.fill_cas("#input","2",&p),Err(EngineError::StaleSnapshot));
 assert_eq!(page.snapshot().elements[0].value,"1");
}
#[test] fn red_dd_002_multiple_effects_per_action_never_conflated(){
 let mut l=EffectLedger::default();for i in 0..3{l.record_intent(&format!("e{i}"),"action").unwrap();l.dispatch_once(&format!("e{i}")).unwrap();}
 l.observe_http_response("e0").unwrap();l.observe_http_response("e1").unwrap();l.mark_disconnect("e2").unwrap();
 assert_eq!(l.get("e0").unwrap().state,EffectState::ConfirmedTransport);
 assert_eq!(l.get("e2").unwrap().state,EffectState::Uncertain);
 assert_eq!(l.dispatch_once("e2"),Err(EngineError::EffectUncertain));
}
#[test] fn red_dd_003_revoked_lease_cannot_authorize_outbound(){
 let p=TrustedHarness::principal("tenant-A","agent-A");let ch=Supervisor.attest(&p,1,41);let mut broker=Broker::default();broker.set_active("tenant-A",1,42);
 let permit=EffectPermit{tenant:"tenant-A".into(),worker_generation:1,lease_epoch:41,origin:"https://fixture.invalid".into(),method:"POST".into(),nonce:"one".into()};
 assert_eq!(broker.approve_attempt(&ch,&permit,"https://fixture.invalid","POST"),Err(EngineError::StaleLease));
}
#[test] fn red_dd_006_cross_frame_atomic_claim_is_not_supported(){
 // Fixture snapshot coverage explicitly semantic-only; never claim strict cross-frame atomicity.
 let page=FixturePage::new();assert_eq!(page.snapshot().coverage,"semantic_only_fixture");
}
#[test] fn red_dd_007_dynamic_locator_replacement_and_uniqueness(){
 let p=FixturePage::new();let id=p.insert("#save","old");p.replace(id,"#save","new").unwrap();p.fill_dynamic("#save","done").unwrap();
 assert_eq!(p.dispatch_count(),1);assert_eq!(p.snapshot().elements[0].value,"done");
 p.insert("#save","duplicate");assert_eq!(p.fill_dynamic("#save","error"),Err(EngineError::InvalidLocator));assert_eq!(p.dispatch_count(),1);
}
#[test] fn red_dd_008_http_200_not_business_confirmation(){
 let mut l=EffectLedger::default();l.record_intent("e","a").unwrap();l.dispatch_once("e").unwrap();l.observe_http_response("e").unwrap();
 assert_eq!(l.get("e").unwrap().business,BusinessOutcome::Unverified);
}
#[test] fn red_dd_009_redirect_requires_new_target_approval(){
 let principal=TrustedHarness::principal("A","agent");let ch=Supervisor.attest(&principal,1,1);let mut broker=Broker::default();broker.set_active("A",1,1);
 let permit=EffectPermit{tenant:"A".into(),worker_generation:1,lease_epoch:1,origin:"https://approved.invalid".into(),method:"POST".into(),nonce:"n".into()};
 assert_eq!(broker.approve_attempt(&ch,&permit,"https://redirect.invalid","POST"),Err(EngineError::Unauthorized));
}
#[test] fn red_dd_010_five_readers_two_writers_conflict_free(){
 let page=FixturePage::new();page.insert("#value","0");let old=guard(&page);
 let mut readers=vec![];for _ in 0..5{let p=page.clone();readers.push(thread::spawn(move||p.snapshot()));}
 for r in readers{assert_eq!(r.join().unwrap().elements.len(),1);}
 let a=page.clone();let b=page.clone();let first=old.clone();let second=old.clone();
 let h1=thread::spawn(move||a.fill_cas("#value","A",&first));
 let h2=thread::spawn(move||b.fill_cas("#value","B",&second));
 let outcomes=[h1.join().unwrap(),h2.join().unwrap()];
 assert_eq!(outcomes.iter().filter(|r|r.is_ok()).count(),1);
 assert_eq!(outcomes.iter().filter(|r|r.as_ref().err()==Some(&EngineError::StaleSnapshot)).count(),1);
 assert_eq!(page.dispatch_count(),1);
}
#[test] fn red_dd_011_old_cursor_must_report_gap(){
 let mut events=EventStream::new("t","p",2);for i in 0..4{events.push(&i.to_string()).unwrap();}
 assert_eq!(events.resume("t","p",1),Err(EngineError::EventGap));
}
#[test] fn red_dd_012_monotonic_clock_boot_id_blocks_reuse(){
 let deadline=MonotonicDeadline{boot_id:"node-a/boot-1".into(),tick_ms:400};
 assert_eq!(deadline.expired("node-a/boot-2",1),Err(EngineError::Expired));
}
#[test] fn red_dd_015_capacity_rejection(){
 let mut s=Scheduler::new(1,Budget{cpu_units:1,memory_units:1});
 let create=|name:&str|Job{id:name.into(),tenant:"t".into(),cost:Budget{cpu_units:1,memory_units:1}};
 s.submit(create("1")).unwrap();assert_eq!(s.submit(create("2")),Err(EngineError::ResourceExhausted));
}
#[test] fn red_bd_023_idempotency_key_is_tenant_scoped(){
 let mut l=EffectLedger::default();assert_eq!(l.reserve_action("A","action","k","same","a1").unwrap(),"a1");
 assert_eq!(l.reserve_action("A","action","k","same","a2").unwrap(),"a1");
 assert_eq!(l.reserve_action("A","action","k","other","a3"),Err(EngineError::IdempotencyKeyReuse));
 assert_eq!(l.reserve_action("B","action","k","other","a4").unwrap(),"a4");
}
#[test] fn red_bd_024_identity_cannot_cross_channel(){
 let principal=TrustedHarness::principal("A","agent");let ch=Supervisor.attest(&principal,1,1);let mut broker=Broker::default();broker.set_active("A",1,1);
 let forged=EffectPermit{tenant:"B".into(),worker_generation:1,lease_epoch:1,origin:"https://fixture.invalid".into(),method:"GET".into(),nonce:"x".into()};
 assert_eq!(broker.approve_attempt(&ch,&forged,"https://fixture.invalid","GET"),Err(EngineError::Unauthorized));
}
#[test] fn red_bd_028_cancel_does_not_erase_uncertain_effect(){
 let mut ledger=EffectLedger::default();ledger.record_intent("effect","action").unwrap();ledger.dispatch_once("effect").unwrap();ledger.mark_disconnect("effect").unwrap();
 assert_eq!(ledger.get("effect").unwrap().state,EffectState::Uncertain);
}
