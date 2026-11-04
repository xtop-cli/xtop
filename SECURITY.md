# Security policy

## Supported versions

The project is pre-1.0. Only the latest `0.1.x` release and the current
`main` branch receive security fixes.

## Reporting a vulnerability

Do not open a public issue for a security problem.

Use GitHub private vulnerability reporting:

https://github.com/xtop-cli/xtop/security/advisories/new

Include the affected version or commit, the platform, reproduction steps and
the impact. Do not include secrets, tokens, private keys or personal data.

## Security model

- Plugins are compile-time Rust crates. Reads and writes go through
  `PluginContext`, which enforces the capabilities declared in the plugin
  manifest. A bypass of these checks is a security defect.
- The MCP server (`xtop mcp`) is a local stdio process. It opens no network
  port and has no authentication layer; do not expose it to untrusted
  clients.
- External runtime widgets run helper processes with the user's privileges.
  The process boundary is the only isolation; they are not sandboxed.
- WASM runtime widgets are constrained by wasmi fuel, a memory cap, a single
  allowed host import and rectangle clipping, but they still share the host
  process.
- Process termination is limited to processes owned by the current user.
- Hardware data is never fabricated; unavailable sources produce honest
  empty states.

## Release verification

Release archives are published with `SHA256SUMS`. The checksums file is
signed keyless with cosign and the archives carry a build provenance
attestation. Verify before installing:

```
cosign verify-blob \
  --bundle SHA256SUMS.bundle \
  --certificate-identity "https://github.com/xtop-cli/xtop/.github/workflows/release.yml@refs/tags/v0.1.0" \
  --certificate-oidc-issuer "https://token.actions.githubusercontent.com" \
  SHA256SUMS
sha256sum -c SHA256SUMS
```
