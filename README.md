# Routex402 Aomi App

A **read-only** Aomi App that lets an agent check a seller's verified payment reputation **before paying**.

Routex402 records payment reputation for AI agents on Solana. A seller's score only changes when the buyer's own wallet signs the update, so a seller cannot inflate its own record.

> Independent project. Not an Aomi product; no partnership is implied. Solana **devnet** data only.

## Tools

| Tool | Purpose |
| --- | --- |
| `check_agent_reputation(seller, policy?)` | Looks up a seller by profile pubkey or payout address. Returns score, tier, confidence, vetting checks and a `proceed` / `do_not_pay` recommendation with reasons. `policy` is `standard` (default) or `strict`. |
| `list_top_agents(limit?)` | Lists sellers ranked by reputation (default 5, max 20). |

The App requests **no host namespaces**: no wallet access and no signing. It cannot move funds.

## Policies

- **standard**: reputation of 80 or more, and buyer independence must not fail (a high score built on one buyer is blocked).
- **strict**: standard, plus confidence of Medium or High.

## Data source

Reads `GET https://route402-seller.onrender.com/agents`. The host runs on a free tier and can take up to ~50 seconds to respond after idling, so the client timeout is 70 seconds.

## Build and test

```bash
cargo test
cargo build --release
```

Requires Rust 1.85+. `aomi-sdk` is pinned exactly (`=7.0.0`) as the Aomi build tooling requires.

## Status

- Compiles and passes unit tests (vetting logic, policy decisions, JSON parsing).
- Not yet deployed to the Aomi platform.
- Unverified: whether App tools may make outbound HTTPS calls from the Aomi runtime, and tool timeout limits.

## Links

- Live demo: https://routex402.xyz/demo.html
- Integration notes: https://routex402.xyz/integrate.html
- Program (devnet): https://solscan.io/account/6pp5jDS23FBcwtqe8pTiBJw3osyHZQPENr9bK3P62vjt?cluster=devnet
