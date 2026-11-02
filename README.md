# farol

*Farol* is Portuguese for "lighthouse": it keeps telling the world where you
are. farol keeps DNS A records pointed at the machine's current public IP, so a
home server stays reachable when the ISP changes the address.

It runs once and exits. Schedule it with a systemd timer or cron.

- **DNS providers:** Hostinger.
- **Public IP source:** [ipify](https://www.ipify.org).
- **Records:** IPv4 (A) only.

## Install

```sh
cargo install --path crates/app-cli
```

This installs the `farol` binary into `~/.cargo/bin`.

## Configure

1. Create a Hostinger API token in hPanel under **Profile → API**.
2. Export it. farol never reads the token from the config file:

   ```sh
   export FAROL_HOSTINGER_TOKEN=...
   ```

3. Write a config listing the records to keep pointed at this machine. See
   [`examples/config.toml`](examples/config.toml):

   ```toml
   provider = "hostinger"

   [[records]]
   domain = "example.com"
   name = "home"   # use "@" for example.com itself
   ttl = 300
   ```

farol looks for the config in this order:

1. The path given with `--config`.
2. `$XDG_CONFIG_HOME/farol/config.toml`.
3. `~/.config/farol/config.toml`.

## Use

```sh
farol check   # show which records have drifted, change nothing
farol sync    # point every drifted record at the public IP
```

```
$ farol sync
public IP 203.0.113.7
example.com: unchanged
home.example.com: pointed (was 198.51.100.4)
```

A record has **drifted** when its A records are anything other than exactly
the public IP: missing, different, or several. Pointing a record replaces all
of its A records with a single one holding the public IP and the configured
TTL. Other record types and other names are left alone.

### Exit status

| Status | Meaning |
|--------|---------|
| `0` | Every record is unchanged, drifted (`check`) or pointed (`sync`). |
| `1` | A record failed, the public IP couldn't be found, or the config or token is missing or invalid. Details go to stderr. |

## Run on a schedule with systemd

[`contrib/systemd`](contrib/systemd) has a hardened service and a timer that
runs `farol sync` every five minutes. The service runs as an unprivileged
dynamic user with no write access anywhere.

```sh
sudo install -m 755 ~/.cargo/bin/farol /usr/local/bin/farol
sudo install -d /etc/farol
sudo install -m 644 examples/config.toml /etc/farol/config.toml   # then edit it
sudo install -m 600 /dev/null /etc/farol/farol.env
echo 'FAROL_HOSTINGER_TOKEN=...' | sudo tee /etc/farol/farol.env >/dev/null
sudo install -m 644 contrib/systemd/farol.service contrib/systemd/farol.timer /etc/systemd/system/
sudo systemctl daemon-reload
sudo systemctl enable --now farol.timer
```

To check that it's working:

```sh
systemctl list-timers farol.timer
journalctl -u farol.service
```

Hostinger allows 90 API requests per minute. farol reads each domain's zone
once per run, so a five-minute schedule stays far below that limit.

### Or with cron

```cron
*/5 * * * * FAROL_HOSTINGER_TOKEN=... /usr/local/bin/farol sync
```

## Development

farol uses stable Rust, pinned in `rust-toolchain.toml`. See
[`CLAUDE.md`](CLAUDE.md) for the engineering rules: Clean Architecture with
vertical slices, lightweight DDD, Clean Code and Canon TDD.

```sh
cargo test --workspace
cargo clippy --workspace --all-targets
cargo llvm-cov --workspace
```

| Crate | Role |
|-------|------|
| `lib-core` | Entities, use cases (check, sync) and gateway traits |
| `lib-ipify` | Finds the public IP through ipify |
| `lib-hostinger` | Reads and points A records through Hostinger's DNS API |
| `app-cli` | The `farol` command: config, output and wiring |

A new DNS provider is a new slice crate that implements `DnsGateway`.

## Licence

MIT
