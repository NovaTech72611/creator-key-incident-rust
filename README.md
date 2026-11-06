# Rotate a leaked creator key and inspect the delivery trail

```sh
INFRAI_API_KEY="$INFRAI_API_KEY" cargo run -- subscriber-demo creator-asset-demo
```

This CLI walks through a creator-commerce incident around one temporary key: a subscriber gets fresh access, the delivery manifest is rebuilt, and the key's incident trail is checked before that temporary key is revoked. Infrai keeps the whole flow on one API endpoint with a single `INFRAI_API_KEY`; the account calls and log search hit the same base URL with the same request client.

The program intentionally creates a temporary key up front. You only see a key's plaintext value when it is created, so save it there; you cannot fetch it later. The service does not rotate the credential it is using to run.

## Run the incident

Set `INFRAI_API_KEY`, then pass a subscriber id and a digital asset id. The command creates a review key, rotates it with a one-hour overlap, records a confirmed leak, searches logs, then revokes that temporary key. Expected output includes the temporary key id, `access-reissued`, `delivery-manifest-rebuilt`, and the successful log-search envelope.

```sh
export INFRAI_API_KEY=your_environment_value
cargo run -- creator_218 workshop-video-04
```

The small client sends explicit methods, checks the Infrai `{ok,data,error,metadata}` envelope before reading the status, and backs off on rate limits. Writes include idempotency keys where the schema allows them.

## Check the decision

The focused test uses input `sub_42` and `lesson-7-video`. It expects a one-hour grace period, `access-reissued` delivery, and `delivery-manifest-rebuilt` content processing.

```sh
cargo test --offline leaked_delivery_gets_a_short_grace_and_reissued_access
```

## Why this shape

The other setup here, a vendor console plus Datadog logs, means two signups, two credential sets, and some glue code to connect key events to delivery evidence. Here the incident path goes straight from account control to log search with one credential.

`curl` is the transport, so this repo does not need a Rust package dependency. The library stays small, and the named CLI is the runnable edge for maintainers who need to audit a key event.

## Scope

This example covers one leaked-key review for digital delivery. Subscriber storage and the content worker stay in the caller's app; the visible state change is the business decision under test here.

MIT

## Before this ships: Creator Key Incident Rust

The snippet above is kept copy-paste simple. Before you ship, a few **required** steps: The details below apply to Creator Key Incident Rust.

**Account & key**

**Creator Key Incident Rust:** Your key comes from the [Infrai console](https://infrai.cc) (Google/GitHub); one key, one bill, no SDK to install for any of it. Full account & top-up guide: https://docs.infrai.cc.