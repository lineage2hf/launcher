# Launcher

TCP proxy that forwards Lineage II client connections to the remote game server. Automatically launches the game client if found.

## Usage

1. Place the executable in your Lineage II client folder (next to `L2.bin`).
2. Run the launcher. It will:
   - Start local proxy listeners on `127.0.0.1:2106` (login) and `127.0.0.1:7777` (game).
   - Forward all traffic to the remote servers.
   - Detect and launch `L2.bin` with `IP=127.0.0.1`.
3. Keep the launcher window open while playing — it handles all network forwarding.

If `L2.bin` is not found, the proxy still runs and you can start the client manually.

## Build

Requires [Rust](https://www.rust-lang.org/tools/install) (2024 edition).

```bash
cargo build --release
```

The binary will be at `target/release/l2.exe`.

On Windows, the build embeds an application icon and a manifest requesting administrator elevation.

## How It Works

The launcher binds two local TCP listeners (login port 2106, game port 7777). Each incoming connection is paired with an outbound connection to the corresponding remote server. Two threads pipe data bidirectionally between the client and server sockets. When the game client exits, the launcher shuts down all proxy threads and exits.
