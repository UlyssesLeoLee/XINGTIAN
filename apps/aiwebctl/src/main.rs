use aiweb_core::action::FixturePage;
use aiweb_core::capabilities::{Support,CAPABILITIES};
use aiweb_core::state::{Dependency,Precondition};
use std::env;
fn main(){
 let cmd=env::args().nth(1).unwrap_or_else(||"help".into());
 match cmd.as_str(){
  "capabilities"=>{for (name,status) in CAPABILITIES {let s=match status{Support::PocFixtureOnly=>"fixture_poc_only",Support::Unsupported=>"unsupported"};println!("{name}: {s}");}},
  "demo"=>{let page=FixturePage::new();page.insert("#query","old");let s=page.snapshot();let p=Precondition{expected:s.version,read_footprint:vec![Dependency::FormValue,Dependency::DomTree,Dependency::Policy]};page.fill_cas("#query","agent-A",&p).expect("first action");let second=page.fill_cas("#query","agent-B",&p);println!("native_fixture_demo={{first:accepted,second:{second:?},dispatch_count:{}}}",page.dispatch_count());},
  _=>{eprintln!("AIWEB controlled fixture POC (not a browser). Usage: aiwebctl capabilities | demo");if cmd!="help"{std::process::exit(2);}}
 }
}
