//! This manifest is normative for what the code REALLY implements.
#[derive(Clone,Copy,Debug,Eq,PartialEq)]pub enum Support{PocFixtureOnly,Unsupported}
pub const CAPABILITIES:&[(&str,Support)]=&[
 ("task_graph_topological_sort",Support::PocFixtureOnly),
 ("bounded_in_memory_scheduler",Support::PocFixtureOnly),
 ("semantic_fixture_snapshot",Support::PocFixtureOnly),
 ("fixture_strict_cas",Support::PocFixtureOnly),
 ("fixture_dynamic_locator_no_retry",Support::PocFixtureOnly),
 ("effect_ledger_in_memory",Support::PocFixtureOnly),
 ("outbound_authorization_simulation",Support::PocFixtureOnly),
 ("html5_parser",Support::Unsupported),
 ("javascript_execution",Support::Unsupported),
 ("web_agent_event_loop",Support::Unsupported),
 ("css_layout_and_geometry",Support::Unsupported),
 ("gpu_raster_screenshot",Support::Unsupported),
 ("network_broker_live",Support::Unsupported),
 ("os_sandbox",Support::Unsupported),
 ("durable_storage",Support::Unsupported),
 ("sse_http_server",Support::Unsupported),
 ("native_playwright_driver",Support::Unsupported),
 ("internet_navigation",Support::Unsupported),
];
