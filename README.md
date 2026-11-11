# farol

*Farol* is Portuguese for "lighthouse": it keeps telling the world where you
are. farol keeps DNS A records pointed at the machine's current public IP, so a
home server stays reachable when the ISP changes the address.

It runs once and exits. Schedule it with a systemd timer or cron.

- **DNS providers:** Hostinger.
- **Public IP source:** [ipify](https://www.ipify.org).
- **Records:** IPv4 (A) only.

## Install

Straight from the repository, without cloning it first:

```sh
cargo install --locked --git https://github.com/GabrielVander/farol app-cli
```

Add `--tag <tag>` or `--rev <commit>` to pin a version instead of taking the
latest `main`. Run the same command with `--force` to upgrade.

Or from a local clone:

```sh
cargo install --locked --path crates/app-cli
```

Either way, this installs the `farol` binary into `~/.cargo/bin`.

## Configure

1. Create a Hostinger API token in hPanel under **Profile → API**.
2. Export it. The token is not an argument because other users on the machine
   can read a process's arguments:

   ```sh
   export FAROL_HOSTINGER_TOKEN=...
   ```

## Use

```sh
farol check --provider hostinger --record example.com:home   # show drift, change nothing
farol sync --provider hostinger --record example.com:home    # point drifted records at the public IP
```

| Option | Meaning |
|--------|---------|
| `--provider` | The DNS provider. Required. `hostinger` is the only one so far. |
| `--record DOMAIN:NAME` | A record to keep pointed at this machine. Required and repeatable. Use `@` as the name for the domain itself, and `*` or `*.home` for a wildcard. Quote wildcards so the shell leaves them alone: `--record 'example.com:*'`. |
| `--ttl SECONDS` | The TTL every tracked record should have. Defaults to 300, so a record with any other TTL counts as drifted unless you pass its value here. |

```
$ farol sync --provider hostinger --record example.com:@ --record example.com:home
public IP 203.0.113.7
example.com: unchanged
home.example.com: pointed (was 198.51.100.4, TTL 300)
```

A record has **drifted** when its A records are anything other than exactly
the public IP (missing, different, or several) or its TTL differs from
`--ttl`. Pointing a record replaces all
of its A records with a single one holding the public IP and the `--ttl` value.
Other record types and other names are left alone.

### Exit status

| Status | Meaning |
|--------|---------|
| `0` | Every record is unchanged, drifted (`check`) or pointed (`sync`). |
| `1` | A record failed, the public IP couldn't be found, a record or TTL is invalid, or the token is missing. Details go to stderr. |
| `2` | The command line is wrong, for example no `--record`. |

## Run on a schedule with systemd

[`contrib/systemd`](contrib/systemd) has a hardened service and a timer that
runs `farol sync` every five minutes. The service runs as an unprivileged
dynamic user with no write access anywhere. Edit the `--record` values in
`farol.service` before installing it.

```sh
sudo install -m 755 ~/.cargo/bin/farol /usr/local/bin/farol
sudo install -d /etc/farol
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
FAROL_HOSTINGER_TOKEN=...
*/5 * * * * /usr/local/bin/farol sync --provider hostinger --record example.com:home
```

## Architecture

The diagrams follow the [C4 model](https://c4model.com), zooming in one level
at a time. The crate diagram under [Development](#development) is the third
level, the components inside the `farol` binary.

**System context.** farol and the people and systems it works with.

```mermaid
flowchart TB
    operator["<b>Home server operator</b><br/>[Person]<br/>Installs farol, chooses the records,<br/>runs check by hand"]
    scheduler["<b>systemd timer / cron</b><br/>[Software System]<br/>Runs farol every few minutes"]
    farol["<b>farol</b><br/>[Software System]<br/>Keeps DNS A records pointed at<br/>the machine's public IP"]
    ipify["<b>ipify</b><br/>[External System]<br/>Reports the caller's public IPv4"]
    hostinger["<b>Hostinger DNS API</b><br/>[External System]<br/>Hosts the domain's DNS zone"]

    operator -- "Runs check / sync,<br/>reads output" --> farol
    scheduler -- "Runs sync" --> farol
    farol -- "Asks for the public IP<br/>[HTTPS]" --> ipify
    farol -- "Reads zones, points A records<br/>[HTTPS, API token]" --> hostinger

    classDef person fill:#08427b,stroke:#052e56,color:#fff
    classDef system fill:#1168bd,stroke:#0b4884,color:#fff
    classDef external fill:#999,stroke:#6b6b6b,color:#fff
    class operator person
    class farol system
    class scheduler,ipify,hostinger external
```

**Containers.** What runs and what stores data when farol is installed with
the systemd units from [`contrib/systemd`](contrib/systemd).

```mermaid
flowchart TB
    operator["<b>Home server operator</b><br/>[Person]"]

    subgraph farolBoundary["farol [Software System]"]
        timer["<b>farol.timer + farol.service</b><br/>[Container: systemd units]<br/>Every five minutes, runs farol sync<br/>as a sandboxed dynamic user"]
        cli["<b>farol</b><br/>[Container: Rust binary]<br/>Compares records with the public IP,<br/>points the drifted ones, then exits"]
        env[("<b>/etc/farol/farol.env</b><br/>[Container: env file, mode 600]<br/>Holds FAROL_HOSTINGER_TOKEN")]
    end

    ipify["<b>ipify</b><br/>[External System]"]
    hostinger["<b>Hostinger DNS API</b><br/>[External System]"]

    operator -- "Runs check / sync<br/>[shell]" --> cli
    operator -- "Installs and edits --record values" --> timer
    timer -- "Starts, with the token<br/>in the environment" --> cli
    env -- "Loaded through EnvironmentFile" --> timer
    cli -- "GET public IP<br/>[HTTPS, ureq]" --> ipify
    cli -- "GET zone, PUT A records<br/>[HTTPS + Bearer token, ureq]" --> hostinger

    classDef person fill:#08427b,stroke:#052e56,color:#fff
    classDef container fill:#438dd5,stroke:#2e6295,color:#fff
    classDef external fill:#999,stroke:#6b6b6b,color:#fff
    class operator person
    class timer,cli,env container
    class ipify,hostinger external
    style farolBoundary fill:none,stroke:#888,stroke-dasharray:5 5
```

## Development

farol tracks stable Rust through `rust-toolchain.toml`. See
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
| `app-cli` | The `farol` command: arguments, output and wiring |

A new DNS provider is a new slice crate that implements `DnsGateway`.

**Components.** The third C4 level: the crates inside the `farol` binary.
Every arrow points inward, toward `lib-core`. The core defines the gateway
traits, the slice crates implement them, and only `app-cli` knows about all of
them.

```mermaid
flowchart LR
    subgraph binary["farol [Container: Rust binary]"]
        cli["<b>app-cli</b><br/>[Component: crate]<br/>The farol command: arguments,<br/>output and wiring"]
        core["<b>lib-core</b><br/>[Component: crate]<br/>Entities, use cases (check, sync)<br/>and gateway traits"]
        ipify["<b>lib-ipify</b><br/>[Component: crate]<br/>Finds the public IP"]
        hostinger["<b>lib-hostinger</b><br/>[Component: crate]<br/>Reads and points A records"]
    end

    ipifyApi["<b>ipify</b><br/>[External System]"]
    hostingerApi["<b>Hostinger DNS API</b><br/>[External System]"]

    cli -- "Runs check / sync" --> core
    cli -- "Wires in" --> ipify
    cli -- "Wires in" --> hostinger
    ipify -- "Implements PublicIpGateway" --> core
    hostinger -- "Implements DnsGateway" --> core
    ipify -- "GET public IP<br/>[HTTPS, ureq]" --> ipifyApi
    hostinger -- "GET zone, PUT A records<br/>[HTTPS + Bearer token, ureq]" --> hostingerApi

    classDef component fill:#85bbf0,stroke:#5d82a8,color:#000
    classDef external fill:#999,stroke:#6b6b6b,color:#fff
    class cli,core,ipify,hostinger component
    class ipifyApi,hostingerApi external
    style binary fill:none,stroke:#888,stroke-dasharray:5 5
```

`lib-core-testing` isn't in the diagram because it isn't in the binary. It
holds the fakes and fixtures the tests share, and only `lib-core` and `app-cli`
depend on it, as a dev-dependency.

### Project layout

```
farol/
├── .github/workflows/ci.yml          # CI: fmt, clippy, tests
├── contrib/systemd/                  # hardened service and five-minute timer
│   ├── farol.service
│   └── farol.timer
├── crates/
│   ├── lib-core/                     # the domain: no I/O, no third-party APIs
│   │   ├── src/
│   │   │   ├── entities/             # domain_name, label, public_ip, record_name, tracked_record, ttl
│   │   │   ├── use_cases/
│   │   │   │   ├── ports/            # dns_gateway, public_ip_gateway
│   │   │   │   ├── check_records.rs
│   │   │   │   ├── sync_records.rs
│   │   │   │   └── report.rs
│   │   │   ├── entities.rs
│   │   │   ├── use_cases.rs
│   │   │   └── lib.rs
│   │   └── tests/                    # check_records, sync_records
│   ├── lib-core-testing/             # fakes and fixtures shared by the tests
│   │   └── src/                      # fakes, fixtures, lib
│   ├── lib-ipify/                    # PublicIpGateway through ipify
│   │   └── src/
│   │       ├── adapters/             # ipify_api, ipify_public_ip_gateway
│   │       └── infrastructure/       # ureq_ipify_api
│   ├── lib-hostinger/                # DnsGateway through Hostinger's DNS API
│   │   └── src/
│   │       ├── adapters/             # hostinger_api, hostinger_dns_gateway, models
│   │       └── infrastructure/       # ureq_hostinger_api
│   └── app-cli/                      # the `farol` binary
│       ├── src/
│       │   ├── adapters/
│       │   │   ├── controllers/      # record_arguments, records_controller
│       │   │   └── presenters/       # record_arguments_view, report_view
│       │   ├── infrastructure/       # command_line, run, token
│       │   ├── lib.rs
│       │   └── main.rs
│       └── tests/farol.rs            # end-to-end tests of the binary
├── Cargo.toml                        # workspace
├── rust-toolchain.toml
├── CLAUDE.md
├── LICENSE
└── README.md
```

## Licence

MIT
