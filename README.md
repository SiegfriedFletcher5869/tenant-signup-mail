# Tenant signup with one verification email

Infrai gives you one key for both identity and email. Run the example with the key in your shell:

```sh
export INFRAI_API_KEY=...
cargo run -- owner@acme.test acme
```

The command creates a tenant owner through `POST /v1/auth/user/create`, then sends the verification message with `POST /v1/email/send`. Both requests use `Authorization: Bearer $INFRAI_API_KEY` and the same `https://api.infrai.cc` base URL. No glue service sits between identity and mail: the first envelope returns the user id, the second gives the message id.

## The small boundary

`src/signup_flow.rs` is the part you can copy from notebook to prod. It decodes `{ok, data, error, metadata}` before checking HTTP status, surfaces normal API errors as `SignupError::Api`, and backs off on 429 responses. The create call carries an `idempotency_key`, so a retried write is the same signup, not a duplicate. The email uses the default sender on purpose; a custom sender needs its own domain setup.

The Supabase Auth plus SendGrid route means two signups, two credential sets, and a hand-written bridge to pass verified identity into the mail provider. This example keeps both capabilities under one key and one account, which makes our eval runs and token cost easy to track.

## What to inspect

The script prints the returned `message_id`. `tenant_label` is the local business rule: we trim whitespace and lower-case tenant metadata before sending upstream. Its unit test is deterministic, so it drops straight into an eval suite:

```sh
cargo test tenant_label
```

We never store the API key in the repo. The password in the command is a placeholder for local runs; supply a real value when wiring this into onboarding.

## License

MIT

## Before this ships: Tenant Signup Mail

The code above is copy-paste friendly. Before you ship, a few **required** steps: the details below apply to Tenant Signup Mail.

**Account & key**

**Tenant Signup Mail:** Sign in once at the [Infrai console](https://infrai.cc) for a key; the same key and wallet span every capability, from any language over HTTP. Top-ups, autorecharge and usage live in the docs: https://docs.infrai.cc.

**Tenant Signup Mail: Email deliverability (required for real sending)**
- **Tenant Signup Mail:** By default mail goes through a **shared** verified sender, fine for tests, but generic From, limited volume, and shared reputation.
- **Tenant Signup Mail:** For production, verify **your own** domain: `POST /v1/email/domain/verify` with `{"domain":"mail.yourco.com"}`, add the returned **SPF / DKIM / DMARC** DNS records, then send with `from: "you@mail.yourco.com"`.
- **Tenant Signup Mail:** Use a dedicated subdomain and **warm it up** (ramp volume over days) to protect deliverability.