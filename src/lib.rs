//! Routex402 Aomi App (read-only).
//!
//! Lets an agent check a seller's verified payment reputation BEFORE paying.
//! No wallet access and no signing: the App requests no host namespaces.
//! Independent project: not an Aomi product, no partnership implied.

use aomi_sdk::{
    DynAomiTool, DynToolCallCtx, dyn_aomi_app,
    schemars::JsonSchema,
    serde_json::{Value, json},
};
use serde::{Deserialize, Serialize};
use std::time::Duration;

const SELLER_API: &str = "https://route402-seller.onrender.com";
const SOLSCAN_ACCOUNT: &str = "https://solscan.io/account/";

#[derive(Clone, Default)]
struct Routex402App;

/// Shape returned by the Routex402 seller API `/agents`.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Agent {
    pub pubkey: String,
    #[serde(default)]
    pub authority: String,
    #[serde(default)]
    pub price: u64,
    pub reputation_score: u32,
    pub completed_count: u64,
    pub failed_count: u64,
    #[serde(default)]
    pub total_volume: u64,
    #[serde(default)]
    pub distinct_buyers: u64,
    #[serde(default)]
    pub last_updated: i64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Status {
    Pass,
    Caution,
    Fail,
    Unknown,
}

#[derive(Debug, Clone, Serialize)]
pub struct Check {
    pub id: &'static str,
    pub label: &'static str,
    pub status: Status,
    pub detail: String,
}

// ---------- pure logic (mirrors the dashboard's vetting.js) ----------

pub fn confidence(a: &Agent) -> &'static str {
    let (c, b) = (a.completed_count, a.distinct_buyers);
    if c < 5 {
        "Unrated"
    } else if b == 0 {
        "Unknown"
    } else if c >= 15 && b >= 5 {
        "High"
    } else if b >= 2 {
        "Medium"
    } else {
        "Low"
    }
}

pub fn tier(a: &Agent) -> &'static str {
    if a.completed_count >= 15 && a.reputation_score >= 90 {
        "Elite"
    } else if a.completed_count >= 5 && a.reputation_score >= 70 {
        "Trusted"
    } else {
        "New"
    }
}

pub fn vetting_checks(a: &Agent, now: i64) -> Vec<Check> {
    let (c, f, b) = (a.completed_count, a.failed_count, a.distinct_buyers);
    let total = c + f;
    let rate = if total == 0 { 0.0 } else { c as f64 / total as f64 * 100.0 };
    let age_days = (now - a.last_updated) as f64 / 86400.0;

    let track = Check {
        id: "track-record",
        label: "Track record",
        status: if c >= 15 { Status::Pass } else if c >= 5 { Status::Caution } else { Status::Fail },
        detail: format!("{c} completed payments{}", if c < 5 { " (not enough history to rate)" } else { "" }),
    };
    let success = Check {
        id: "success-rate",
        label: "Success rate",
        status: if total == 0 {
            Status::Unknown
        } else if rate >= 95.0 {
            Status::Pass
        } else if rate >= 80.0 {
            Status::Caution
        } else {
            Status::Fail
        },
        detail: if total == 0 {
            "No payments recorded yet".into()
        } else {
            format!("{rate:.1}% ({c} ok, {f} failed)")
        },
    };
    let (bstatus, bdetail) = if c > 0 && b == 0 {
        (Status::Unknown, "Distinct buyer count not recorded for this agent".to_string())
    } else if b >= 5 {
        (Status::Pass, format!("{b} distinct buyers"))
    } else if b >= 2 {
        (Status::Caution, format!("Only {b} distinct buyers"))
    } else if b == 1 && c >= 10 {
        (Status::Fail, format!("{c} payments from a single buyer"))
    } else {
        (Status::Unknown, "Too little history to judge".to_string())
    };
    let buyers = Check { id: "buyer-independence", label: "Buyer independence", status: bstatus, detail: bdetail };
    let recency = Check {
        id: "recency",
        label: "Recent activity",
        status: if age_days <= 7.0 { Status::Pass } else if age_days <= 30.0 { Status::Caution } else { Status::Fail },
        detail: format!("Last update {} days ago", age_days.floor() as i64),
    };
    vec![track, success, buyers, recency]
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Policy {
    Standard,
    Strict,
}

impl Policy {
    pub fn parse(s: Option<&str>) -> Policy {
        match s.map(|x| x.to_ascii_lowercase()) {
            Some(ref x) if x == "strict" => Policy::Strict,
            _ => Policy::Standard,
        }
    }
    fn name(self) -> &'static str {
        match self {
            Policy::Standard => "standard",
            Policy::Strict => "strict",
        }
    }
}

/// Returns (allow, reasons-for-blocking).
pub fn decide(a: &Agent, checks: &[Check], policy: Policy) -> (bool, Vec<String>) {
    let mut reasons = vec![];
    if a.reputation_score < 80 {
        reasons.push(format!("Reputation {} is below the minimum of 80", a.reputation_score));
    }
    if let Some(c) = checks.iter().find(|c| c.id == "buyer-independence" && c.status == Status::Fail) {
        reasons.push(c.detail.clone());
    }
    if policy == Policy::Strict && !matches!(confidence(a), "Medium" | "High") {
        reasons.push(format!("Confidence is {}; strict policy needs Medium or High", confidence(a)));
    }
    (reasons.is_empty(), reasons)
}

pub fn now_secs() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

fn describe(a: &Agent, now: i64, policy: Policy) -> Value {
    let checks = vetting_checks(a, now);
    let (allow, reasons) = decide(a, &checks, policy);
    json!({
        "seller": a.pubkey,
        "payoutAccount": a.authority,
        "reputationScore": a.reputation_score,
        "tier": tier(a),
        "confidence": confidence(a),
        "completedPayments": a.completed_count,
        "failedPayments": a.failed_count,
        "distinctBuyers": a.distinct_buyers,
        "checks": checks,
        "policy": policy.name(),
        "recommendation": if allow { "proceed" } else { "do_not_pay" },
        "blockReasons": reasons,
        "evidence": format!("{SOLSCAN_ACCOUNT}{}?cluster=devnet", a.pubkey),
        "note": "Scores move only when the buyer's own wallet signs the update. Devnet data."
    })
}

// ---------- network ----------

fn fetch_agents() -> Result<Vec<Agent>, String> {
    // The seller runs on a free tier that can take ~50s to wake, so allow a long timeout.
    let agent = ureq::Agent::config_builder()
        .timeout_global(Some(Duration::from_secs(70)))
        .build()
        .new_agent();
    let mut resp = agent
        .get(&format!("{SELLER_API}/agents"))
        .call()
        .map_err(|e| format!("Routex402 API unreachable: {e}"))?;
    resp.body_mut()
        .read_json::<Vec<Agent>>()
        .map_err(|e| format!("Unexpected Routex402 API response: {e}"))
}

// ---------- tools ----------

#[derive(Debug, Deserialize, JsonSchema)]
struct CheckArgs {
    /// Seller's Routex402 profile pubkey OR its payout (authority) address, base58.
    seller: String,
    /// "standard" (default) or "strict" (also requires Medium or High confidence).
    policy: Option<String>,
}

struct CheckAgentReputation;

impl DynAomiTool for CheckAgentReputation {
    type App = Routex402App;
    type Args = CheckArgs;
    const NAME: &'static str = "check_agent_reputation";
    const DESCRIPTION: &'static str = "Use BEFORE paying an AI agent or service. Looks up the seller's verified payment reputation on Routex402 (Solana devnet) and returns score, tier, confidence, vetting checks and a proceed / do_not_pay recommendation. Read-only; never moves funds.";

    fn run(_app: &Routex402App, args: CheckArgs, _ctx: DynToolCallCtx) -> Result<Value, String> {
        let agents = fetch_agents()?;
        let key = args.seller.trim();
        let policy = Policy::parse(args.policy.as_deref());
        match agents.iter().find(|a| a.pubkey == key || a.authority == key) {
            Some(a) => Ok(describe(a, now_secs(), policy)),
            None => Ok(json!({
                "seller": key,
                "found": false,
                "recommendation": "do_not_pay",
                "blockReasons": ["No Routex402 record for this seller. It has no verified payment history."],
            })),
        }
    }
}

#[derive(Debug, Deserialize, JsonSchema)]
struct TopArgs {
    /// How many sellers to return (default 5, max 20).
    limit: Option<u32>,
}

struct ListTopAgents;

impl DynAomiTool for ListTopAgents {
    type App = Routex402App;
    type Args = TopArgs;
    const NAME: &'static str = "list_top_agents";
    const DESCRIPTION: &'static str = "Use when the user wants to find a trustworthy seller agent. Lists Routex402 sellers ranked by verified reputation, with tier and confidence. Read-only.";

    fn run(_app: &Routex402App, args: TopArgs, _ctx: DynToolCallCtx) -> Result<Value, String> {
        let mut agents = fetch_agents()?;
        agents.sort_by(|a, b| b.reputation_score.cmp(&a.reputation_score).then(b.completed_count.cmp(&a.completed_count)));
        let n = args.limit.unwrap_or(5).clamp(1, 20) as usize;
        let list: Vec<Value> = agents
            .iter()
            .take(n)
            .map(|a| {
                json!({
                    "seller": a.pubkey,
                    "reputationScore": a.reputation_score,
                    "tier": tier(a),
                    "confidence": confidence(a),
                    "completedPayments": a.completed_count,
                    "distinctBuyers": a.distinct_buyers,
                })
            })
            .collect();
        Ok(json!({ "agents": list, "total": agents.len(), "note": "Call check_agent_reputation before paying any of these." }))
    }
}

const PREAMBLE: &str = "You help users pay AI agents safely using Routex402 verified reputation (Solana devnet). \
Before the user pays any seller agent, call check_agent_reputation. If the recommendation is do_not_pay, \
explain the block reasons plainly and do not proceed unless the user explicitly overrides after hearing them. \
If the user has no seller in mind, call list_top_agents first. Explain what Confidence means: Unknown or Low \
means the evidence is thin or comes from few buyers. Never claim certainty: reputation is evidence, not a guarantee. \
This App is read-only and cannot move funds.";

dyn_aomi_app!(
    app = Routex402App,
    name = "routex402",
    version = "0.1.0",
    preamble = PREAMBLE,
    tools = [CheckAgentReputation, ListTopAgents],
    namespaces = []
);

#[cfg(test)]
mod tests {
    use super::*;

    fn agent(rep: u32, done: u64, failed: u64, buyers: u64, updated: i64) -> Agent {
        Agent {
            pubkey: "P".into(), authority: "A".into(), price: 10000, reputation_score: rep,
            completed_count: done, failed_count: failed, total_volume: 0, distinct_buyers: buyers,
            last_updated: updated,
        }
    }

    #[test]
    fn single_buyer_agent_is_blocked() {
        let a = agent(100, 45, 0, 1, 1000);
        let (ok, why) = decide(&a, &vetting_checks(&a, 1000), Policy::Standard);
        assert!(!ok);
        assert!(why[0].contains("single buyer"));
    }

    #[test]
    fn diverse_agent_is_allowed() {
        let a = agent(95, 30, 0, 8, 1000);
        assert!(decide(&a, &vetting_checks(&a, 1000), Policy::Strict).0);
        assert_eq!(confidence(&a), "High");
        assert_eq!(tier(&a), "Elite");
    }

    #[test]
    fn unknown_buyers_pass_standard_but_not_strict() {
        let a = agent(94, 20, 0, 0, 1000);
        assert_eq!(confidence(&a), "Unknown");
        assert!(decide(&a, &vetting_checks(&a, 1000), Policy::Standard).0);
        assert!(!decide(&a, &vetting_checks(&a, 1000), Policy::Strict).0);
    }

    #[test]
    fn stale_agent_fails_recency() {
        let a = agent(95, 30, 0, 8, 0);
        let c = vetting_checks(&a, 120 * 86400);
        assert_eq!(c.iter().find(|x| x.id == "recency").unwrap().status, Status::Fail);
    }

    #[test]
    fn parses_live_agent_json() {
        let j = r#"[{"pubkey":"3y5N","authority":"5gse","price":10000,"reputationScore":100,"completedCount":45,"failedCount":0,"totalVolume":455911,"distinctBuyers":1,"lastUpdated":1783963814}]"#;
        let v: Vec<Agent> = serde_json::from_str(j).unwrap();
        assert_eq!(v[0].completed_count, 45);
    }
}
