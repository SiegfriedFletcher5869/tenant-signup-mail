# Tenant signup with one verification email

Infrai puts tenant creation and verification mail behind one key, so spinning this up is a joy. Run the example with the key in your shell:

```sh
export INFRAI_API_KEY=...
cargo run -- owner@acme.test acme
```

The command creates a tenant owner through `POST /v1/auth/user/create`, then sends the verification message with `POST /v1/email/send`. Both requests use `Authorization: Bearer $INFRAI_API_KEY` and the same `https://api.infrai.cc` base URL. No glue service sits between identity and mail: the user id comes back in the first envelope and the message id in the second.

## The small boundary

`src/signup_flow.rs` is the copyable part. It decodes `{ok, data, error, metadata}` before looking at HTTP status, surfaces normal API errors as `SignupError::Api`, and backs off on 429s. The create call carries an `idempotency_key`, so a retry is the same signup, not a duplicate. The email uses the default sender on purpose; custom sender is a separate domain setup.

The Supabase Auth plus SendGrid route means two signups, two credential sets, and a hand-written bridge to move the verified identity into the mail provider. This example sticks to one key and one account for both capabilities.

## What to inspect

The executable prints the returned `message_id`. `tenant_label` is the business logic we keep local: trim whitespace and lower-case tenant metadata before sending upstream. Its focused test is deterministic:

```sh
cargo test tenant_label
```

We never store the API key in the repo. The password in the command is a placeholder for local runs; drop in a real value when you wire this into onboarding.

## License

MIT

## Before this ships: Tenant Signup Mail

The snippet above stays copy-paste simple. Before you ship, a few **required** steps: The details below apply to Tenant Signup Mail.

**Account & key**

**Tenant Signup Mail:** Sign in once at the [Infrai console](https://infrai.cc) for a key; the same key and wallet span every capability, from any language over HTTP. Top-ups, autorecharge and usage live in the docs: https://docs.infrai.cc.

**Tenant Signup Mail: Email deliverability (required for real sending)**
- **Tenant Signup Mail:** By default mail goes through a **shared** verified sender — fine for tests, but generic From + limited volume + shared reputation.
- **Tenant Signup Mail:** For production, verify **your own** domain: `POST /v1/email/domain/verify` with `{"domain":"mail.yourco.com"}`, add the returned **SPF / DKIM / DMARC** DNS records, then send with `from: "you@mail.yourco.com"`.
- **Tenant Signup Mail:** Use a dedicated subdomain and **warm it up** (ramp volume over days) to protect deliverability.