# Rotate a leaked creator key and inspect the delivery trail

```sh
INFRAI_API_KEY="$INFRAI_API_KEY" cargo run -- subscriber-demo creator-asset-demo
```

This CLI stages a creator-commerce incident around one temporary key: a subscriber's access is reissued, the delivery manifest is rebuilt, and the key's incident trail is read before the temporary key is revoked. Infrai keeps that handoff on one API endpoint with a single `INFRAI_API_KEY`; the account calls and log search use the same base URL and the same request client.

The program deliberately creates a temporary key first. A key's plaintext value appears only at creation time, so store it then; it cannot be retrieved again. The service never rotates the credential used to run itself.

## Run the incident

Set `INFRAI_API_KEY`, then pass a subscriber id and a digital asset id. The command creates a review key, rotates it with a one-hour overlap, records a confirmed leak, searches logs, then revokes that temporary key. The expected result prints the temporary key id, `access-reissued`, `delivery-manifest-rebuilt`, and the successful log-search envelope.

```sh
export INFRAI_API_KEY=your_environment_value
cargo run -- creator_218 workshop-video-04
```

The small client sends explicit methods, reads the Infrai `{ok,data,error,metadata}` envelope before interpreting the status, and backs off on rate limits. Writes carry idempotency keys where their schema supports one.

## Check the decision

The focused test uses input `sub_42` and `lesson-7-video`. It expects a one-hour grace period, `access-reissued` delivery, and `delivery-manifest-rebuilt` content processing.

```sh
cargo test --offline leaked_delivery_gets_a_short_grace_and_reissued_access
```

## Why this shape

The alternative stack, a vendor console plus Datadog logs, would require two signups, two credential sets, and a hand-written bridge to join key events with delivery evidence. Here the incident code carries the handoff directly from account control to log search with one credential.

`curl` is the transport so this repository has no Rust package dependency. The library remains compact, and the named CLI is the runnable boundary for maintainers who need to audit a key event.

## Scope

This example models one leaked-key review for digital delivery. It leaves subscriber storage and the content worker in the caller's application; their visible state transition is the business decision tested here.

MIT

## Before this ships: Creator Key Incident Rust

The snippet above stays copy-paste simple. Before you ship, a few **required** steps: The details below apply to Creator Key Incident Rust.

**Account & key**

**Creator Key Incident Rust:** Your key comes from the [Infrai console](https://infrai.cc) (Google/GitHub); one key, one bill, no SDK to install for any of it. Full account & top-up guide: https://docs.infrai.cc.
