//! Fixture-only volume checks; not browser, sandbox, leak, or 60-minute ST evidence.
use aiweb_core::action::FixturePage;
use std::thread;
#[test]
#[ignore = "fixture-only context loop; run explicitly, do not count as OS sandbox ST"]
fn fixture_1000_context_lifetimes() {
 for i in 0..1000 {
   let p=FixturePage::new();p.insert("#q",&format!("tenant-token-{i}"));
   assert_eq!(p.snapshot().elements[0].value,format!("tenant-token-{i}"));
   drop(p);
 }
 let next=FixturePage::new();assert!(next.snapshot().elements.is_empty());
}
#[test]
#[ignore = "fixture-only parallel reads; not browser concurrency performance evidence"]
fn fixture_8_16_32_reader_concurrency() {
 for threads in [8,16,32] {
   let page=FixturePage::new();page.insert("#q","unchanged");
   let handles:Vec<_>=(0..threads).map(|_|{let p=page.clone();thread::spawn(move || {
     for _ in 0..1000 {assert_eq!(p.snapshot().elements[0].value,"unchanged");}
   })}).collect();
   for h in handles {h.join().unwrap();}
 }
}
