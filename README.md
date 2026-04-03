# OpenTTD Server

A Dockerised OpenTTD dedicated server exposed via a [playit.gg](https://playit.gg) tunnel, with a TCP/UDP proxy (tcp-guard) sitting between the tunnel and the game server.

## Architecture

```
Internet → playit tunnel → tcp-guard (TCP_GUARD_IP:3979) → openttd-server (OPENTTD_SERVER_IP:3979)
```

## Prerequisites

- Docker and Docker Compose
- A [playit.gg](https://playit.gg) account with a tunnel configured for port 3979

## Setup

### 1. Configure the playit.gg tunnel

In the playit.gg dashboard, set the tunnel's local destination to:

```
172.19.0.10:3979
```

> If you change `TCP_GUARD_IP` in your `.env`, use that IP here instead.

### 2. Create a `.env` file

Copy the example below and fill in your values:

```env
# Path to your .openttd directory on the host (required)
OPENTTD_DATA_DIR=/path/to/your/.openttd

# Your UID and GID so the container user matches the host user (for file permissions)
PUID=1000
PGID=1000

# Your playit.gg agent secret key (required)
PLAYIT_SECRET_KEY=your_secret_key_here

# Admin port password — synced to secrets.cfg automatically by run.sh
OPENTTD_ADMIN_PASSWORD=your_admin_password_here

# Server name shown in the multiplayer lobby — synced to openttd.cfg automatically by run.sh
SERVER_NAME="My OpenTTD Server"

# Load a saved game on startup (optional)
# Set to "true" and provide SAVENAME to load a specific save
# Set to "last-autosave" to load the most recent autosave
# Set to "exit" to load the exit save
# LOADGAME=true
# SAVENAME=game.sav
```

### 3. Start the server

```bash
./run.sh
```

The script:

1. Validates that all required environment variables are set
2. **Creates or updates `openttd.cfg` and `secrets.cfg`** inside `OPENTTD_DATA_DIR`:
   - If either file is missing it is created with sensible defaults
   - `allow_insecure_admin_login = true` is enforced in `openttd.cfg` (required for the bot)
   - `SERVER_NAME` (optional) is synced to `server_name` in `private.cfg`
   - `OPENTTD_ADMIN_PASSWORD` is synced to `admin_password` in `secrets.cfg`
   - `SERVER_PASSWORD` (optional) is synced to `server_password` in `secrets.cfg`
3. Builds the tcp-guard image and starts all containers

> **Note:** OpenTTD overwrites `openttd.cfg` on shutdown with its current in-memory state. If you edit the file while the server is running, restart immediately — otherwise the running server will overwrite your changes on exit. Prefer setting values via `.env` so `run.sh` re-applies them on every start.

---

## Configuration File Examples

### `.env`

```env
OPENTTD_DATA_DIR=/home/youruser/.local/share/openttd
PUID=1000
PGID=1000

PLAYIT_SECRET_KEY=your_secret_key_here

LOADGAME=true
SAVENAME=mygame.sav

OPENTTD_ADMIN_PASSWORD=your_admin_password_here
SERVER_PASSWORD=your_join_password_here   # optional — leave unset for a public server
SERVER_NAME="My OpenTTD Server"           # optional — shown in the multiplayer lobby
SAVE_INTERVAL_MINS=10
```

### `openttd.cfg`, `private.cfg`, and `secrets.cfg`

`run.sh` creates and maintains these files automatically based on your `.env`. You do not need to edit them by hand. If a file is missing it is created with working defaults; if it already exists only the values controlled by `.env` are updated.

The bot requires the following settings in `openttd.cfg` under `[network]` — `run.sh` ensures they are always set:

```ini
[network]
server_admin_port = 3977
allow_insecure_admin_login = true
```

`private.cfg` (where OpenTTD 15.x stores the server name and other private settings):

```ini
[network]
server_name = <SERVER_NAME>
```

`secrets.cfg` (where OpenTTD 15.x stores all passwords):

```ini
[network]
server_password = <SERVER_PASSWORD>
rcon_password =
admin_password = <OPENTTD_ADMIN_PASSWORD>
```

> **Note:** OpenTTD overwrites `openttd.cfg` on shutdown. Prefer managing all settings through `.env` so `run.sh` re-applies them on every start.

---

## Environment Variables

### OpenTTD Server

| Variable | Required | Default | Description |
|---|---|---|---|
| `OPENTTD_DATA_DIR` | Yes | — | Host path mounted as `/home/openttd/.openttd`. Must contain your `openttd.cfg` and `save/` directory |
| `PUID` | No | current user | User ID of the `openttd` user inside the container. Defaults to the UID of whoever runs `run.sh`. Override in `.env` if needed |
| `PGID` | No | current user | Group ID of the `openttd` user inside the container. Defaults to the GID of whoever runs `run.sh`. Override in `.env` if needed |
| `LOADGAME` | No | `false` | Controls which save to load: `false` (new game), `true` (use `SAVENAME`), `last-autosave`, or `exit` |
| `SAVENAME` | No | — | Save file name to load when `LOADGAME=true` (e.g. `game.sav`) |

### Utils Bot

| Variable | Required | Default | Description |
|---|---|---|---|
| `OPENTTD_ADMIN_PASSWORD` | Yes | — | Password for the OpenTTD admin port. Synced to `admin_password` in `secrets.cfg` by `run.sh` |
| `SERVER_PASSWORD` | No | — | Join password for the game server. Synced to `server_password` in `secrets.cfg` by `run.sh`. Leave unset for a public server |
| `SERVER_NAME` | No | — | Name shown in the multiplayer lobby. Synced to `server_name` in `private.cfg` by `run.sh` |
| `OPENTTD_ADMIN_PORT` | No | `3977` | Admin port the bot connects to. Must match `server_admin_port` in `openttd.cfg` |
| `BOT_NAME` | No | `utils-bot` | Name the bot presents to the server on the admin connection |
| `SAVENAME` | No | `autosave_bot` | Save file name used by the bot for periodic saves. `.sav` extension is stripped automatically if present |
| `SAVE_INTERVAL_MINS` | No | `10` | How often (in minutes) the bot checks whether to auto-save |

### Playit Tunnel

| Variable | Required | Default | Description |
|---|---|---|---|
| `PLAYIT_SECRET_KEY` | Yes | — | Secret key from the playit.gg dashboard |

### TCP Guard

| Variable | Required | Default | Description |
|---|---|---|---|
| `TCP_GUARD_PROXY_TIMEOUT` | No | `30m` | How long an idle connection is kept open. Use nginx time format: `30m`, `3h`, `1h30m` |
| `TCP_GUARD_CONNECT_TIMEOUT` | No | `5s` | How long to wait for the upstream (openttd-server) to accept a TCP connection |
| `TCP_GUARD_PREREAD_TIMEOUT` | No | `5s` | How long to wait for the client to send data after connecting |
| `TCP_GUARD_MAX_CONN` | No | `16` | Maximum simultaneous TCP connections (acts as a player cap since all traffic arrives from the playit tunnel with a single source IP) |

### Bot Chat Commands

Players can type these in the in-game chat to control the bot:

| Command | Description |
|---|---|
| `!pause` | Pause the game |
| `!unpause` | Unpause the game |
| `!save` | Trigger an immediate save (resets the auto-save timer) |

### Autosave behaviour

The bot saves the game as `SAVENAME` every `SAVE_INTERVAL_MINS` minutes. If the in-game date has not advanced since the last save (e.g. the game is paused), the save is skipped and logged as "Auto-save skipped — in-game date unchanged since last save."

### Docker Network

| Variable | Required | Default | Description |
|---|---|---|---|
| `OPENTTD_SERVER_IP` | No | `172.19.0.5` | Static IP assigned to the openttd-server container |
| `TCP_GUARD_IP` | No | `172.19.0.10` | Static IP assigned to the tcp-guard container. Set this as the tunnel destination in the playit.gg dashboard |
| `DOCKER_SUBNET` | No | `172.19.0.0/24` | Subnet for the internal Docker network. Change if it conflicts with your host network |

---

## Scripts

| Script | Description |
|---|---|
| `run.sh` | Build and start all containers |
| `stop.sh` | Stop the openttd-server container |
| `restart.sh` | Restart the openttd-server container |
| `remove.sh` | Stop and remove all containers |
| `logs.sh` | Tail openttd-server logs |
| `console.sh` | Attach to the openttd-server console |

---

## Troubleshooting

### Players can't connect

1. Check the playit.gg dashboard — the tunnel destination must be `<TCP_GUARD_IP>:3979` (default `172.19.0.10:3979`)
2. Confirm all containers are running: `docker ps`
3. Check tcp-guard logs for refused connections: `docker logs tcp-guard`

### Server can't read/write save files

Check that `PUID` and `PGID` match your host user. Run `id` on the host to find your UID and GID.

### Docker network subnet is already in use

Set `DOCKER_SUBNET` in your `.env` to a free subnet and update `OPENTTD_SERVER_IP` and `TCP_GUARD_IP` to addresses within it. Remove the old network before restarting:

```bash
./remove.sh
docker network prune
./run.sh
```

### Utils bot keeps printing "Connection refused"

The bot connects to the OpenTTD admin port (3977). If it can't connect:

1. **Re-run `run.sh`** — it enforces `allow_insecure_admin_login = true` and syncs the admin password into `secrets.cfg` on every start. If the config was out of date, this should fix it.
2. **Check `secrets.cfg`** — in OpenTTD 15.x, `admin_password` must be set in `secrets.cfg`. An empty password causes OpenTTD to skip opening the admin port entirely.
3. **Check `allow_insecure_admin_login`** — must be `true` in `openttd.cfg`. When `false`, OpenTTD will not open the plain-TCP admin port.
4. **Verify the port is open** — after restarting, `docker logs openttd-server` should include a line like `Listening on 0.0.0.0:3977 (Admin Port)`. If that line is absent, the admin port did not start (check the points above).
5. **Check the password matches** — `OPENTTD_ADMIN_PASSWORD` in `.env` must equal `admin_password` in `secrets.cfg`.

### tcp-guard fails to start

Check logs with `docker logs tcp-guard`. A common cause is an invalid value for one of the `TCP_GUARD_*` timeout variables — nginx requires a specific time format (e.g. `5s`, `30m`, `3h`). Plain numbers without a unit will cause nginx to reject the config.
