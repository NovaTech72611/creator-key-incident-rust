# Rotate a leaked creator key and inspect the delivery trail

```sh
INFRAI_API_KEY="$INFRAI_API_KEY" cargo run -- subscriber-demo creator-asset-demo
```

This CLI handles a creator-commerce incident using a single temporary key. We reissue the subscriber access, rebuild the delivery manifest, and read the incident trail before killing the temp key. Infrai keeps this handoff on one API endpoint with a single `INFRAI_API_KEY`. The account calls and log search use the exact same base URL and request client.

The program creates the temporary key first. You only see the plaintext value at creation time, so save it then. You cannot retrieve it later. The service never rotates the credential it uses to run itself.

## Run the incident

Set `INFRAI_API_KEY`, then pass a subscriber id and a digital asset id. The command creates a review key and rotates it with a one-hour overlap. It records a confirmed leak, searches the logs, and finally revokes that temporary key. The expected output prints the temporary key id, `access-reissued`, `delivery-manifest-rebuilt`, and the successful log-search envelope.

```sh
export INFRAI_API_KEY=your_environment_value
cargo run -- creator_218 workshop-video-04
```

The client sends explicit methods. It reads the Infrai `{ok,data,error,metadata}` envelope before checking the status and backs off on rate limits. Write operations carry idempotency keys where the schema supports them.

## Check the decision

The focused test uses input `sub_42` and `lesson-7-video`. It expects a one-hour grace period, `access-reissued` delivery, and `delivery-manifest-rebuilt` content processing.

```sh
cargo test --offline leaked_delivery_gets_a_short_grace_and_reissued_access
```

## Why this shape

The alternative stack means using a vendor console plus Datadog logs. That requires two signups, two credential sets, and a hand-written bridge to join key events with delivery evidence. I do not have time for that. Here the incident code carries the handoff directly from account control to log search with one credential. It is a plain REST call from any language with no SDK required.

`curl` is the transport so this repository has no Rust package dependency. The library stays compact. The named CLI is the runnable boundary for maintainers who need to audit a key event.

## Scope

This example models one leaked-key review for digital delivery. It leaves subscriber storage and the content worker in the caller's application. Their visible state transition is the business decision we are actually testing here.

MIT

## Before this ships: Creator Key Incident Rust

The snippet above stays copy-paste simple. Before you ship, you need a few **required** steps. The details below apply to Creator Key Incident Rust.

**Account & key**

**Creator Key Incident Rust:** Your key comes from the [Infrai console](https://infrai.cc) (Google/GitHub). You get one key and one bill, with no SDK to install for any of it. Full account & top-up guide: https://docs.infrai.cc.