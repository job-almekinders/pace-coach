# Pace Coach

A macOS CLI daemon that watches your typing rhythm and nudges you when you are rushing/stressed.

It monitors your keystrokes locally (nothing leaves your machine), classifies your typing state every two seconds, fires a macOS notification when you've been in a stressed state for too long, and shows a live emoji in your menu bar.

**States:**

- ⚪ Idle — no recent typing
- 🔵 Passive — low activity or irregular rhythm
- 🟡 Normal — active, correction rate within range
- 🔴 Stressed — high correction rate sustained

---

## Why

I built this to explore Rust and macOS native development, creating a lightweight CLI tool that provides awareness of my workflow patterns.
This is by no means a perfect tool and/or perfect code, but it was a fun way to get acquinted with these new tools :)

---

## Install

Download the latest release, extract both binaries, and put them on your `$PATH`:

```bash
# replace version with your desired version
VERSION=0.1.0
curl -L https://github.com/job-almekinders/pace-coach/releases/download/v$VERSION/pace-coach-$VERSION-aarch64-apple-darwin.tar.gz | tar xz
mv pace-coach pace-coach-menubar /usr/local/bin/
```

pace-coach requires **Input Monitoring** permission (System Settings → Privacy & Security → Input Monitoring). Grant it on first run.

---

## Usage

```bash
pace-coach start             # start daemon + menu bar icon
pace-coach stop              # stop both
pace-coach status            # NORMAL 🟡
pace-coach status --verbose  # full metrics
pace-coach config show       # print current config
```

---

## Configuration

Edit `~/.pace-coach/config.json` then restart the daemon:

```json
{
  "correction_rate_threshold": 0.06,
  "stress_duration_secs": 10,
  "nudge_cooldown_secs": 60
}
```

| Setting                     | Default | Description                                                                                   |
| --------------------------- | ------- | --------------------------------------------------------------------------------------------- |
| `correction_rate_threshold` | `0.06`  | Fraction of keystrokes that are corrections before state is STRESSED. Lower = more sensitive. |
| `stress_duration_secs`      | `10`    | Seconds of sustained STRESSED state before a nudge fires.                                     |
| `nudge_cooldown_secs`       | `60`    | Minimum seconds between nudges.                                                               |

---

## Logs

`~/.pace-coach/pace-coach.log` — daemon stderr. Useful for debugging.

---

## Build from source

Requires Rust stable ([rustup.rs](https://rustup.rs)) and Xcode Command Line Tools (`xcode-select --install`).

**Install both binaries locally:**

```bash
make install
```

This puts `pace-coach` and `pace-coach-menubar` in `~/.cargo/bin/`.

**Build release binaries only:**

```bash
make build
# binaries at target/release/pace-coach and target/release/pace-coach-menubar
```

---

## License

MIT

---

## Contributions

Contributions for fixes or feature additions are welcome! Feature additions I can foresee for this project are:

- A more user friendly installation method, such that non-technical users can also use `pace-coach`.
- A more user friendly method to call the CLI commands e.g. via the symbol in the menu bar.
