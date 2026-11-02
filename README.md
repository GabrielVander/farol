# farol

*Farol* is Portuguese for "lighthouse": it keeps telling the world where you
are. farol keeps DNS A records pointed at the machine's current public IP.

Supported DNS providers: Hostinger.

It runs once and exits, so schedule it with a systemd timer or cron.

## Development

Uses stable Rust (pinned in `rust-toolchain.toml`). See `CLAUDE.md` for the
engineering rules.

```sh
cargo test --workspace
```
