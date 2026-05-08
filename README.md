# aprs-tap

A terminal viewer for [APRS-IS](https://www.aprs-is.net/) streams. Connects to a server, decodes incoming packets in real time, and prints them in a readable format.

## Install

```
cargo install --git https://github.com/displaced-lobster/aprs-tap
```

Or clone and install locally:

```
git clone https://github.com/displaced-lobster/aprs-tap
cd aprs-tap
cargo install --path .
```

## Usage

```
aprs-tap [OPTIONS]
```

### Options

| Flag | Default | Description |
|------|---------|-------------|
| `-s`, `--server` | `rotate.aprs2.net` | APRS-IS server hostname |
| `-p`, `--port` | `14580` | Server port (`14580` = filtered, `10152` = full feed) |
| `-u`, `--callsign` | `N0CALL` | Your callsign (use `N0CALL` for anonymous read-only) |
| `--passcode` | `-1` | APRS-IS passcode (`-1` for read-only) |
| `-f`, `--filter` | — | Server-side filter string |

### Examples

Watch all traffic near Vancouver (100 km radius):
```
aprs-tap -f "r/49.25/-123.1/100"
```

Watch traffic for a specific callsign:
```
aprs-tap -f "b/VA7ASI*"
```

Full unfiltered feed:
```
aprs-tap -p 10152
```

## Packet types decoded

| Type | Description |
|------|-------------|
| Position (`!`, `=`) | Uncompressed and base-91 compressed coordinates, symbol, altitude, comment |
| Position with timestamp (`/`, `@`) | Same as above, with 7-character timestamp stripped |
| Weather (`_`) | Temperature, wind speed/direction, gust, humidity, pressure, rainfall |
| MicE (`` ` ``, `'`) | Compressed position and speed/course encoded in the destination field |
| Object (`;`) | Named object with position, live/killed state, and optional weather or altitude |
| Message (`:`) | Addressed message with sequence number stripped |
| Station capabilities (`<`) | iGate, digipeater, and other capability flags |

Server comment lines (starting with `#`) are shown dimmed. Unrecognised packets are passed through as-is.

## Filter syntax

Filters are applied server-side by the APRS-IS network. Common patterns:

- `r/lat/lon/km` — circle around a point
- `b/CALL*` — by callsign prefix (wildcard `*` supported)
- `t/m` — type filter (e.g. `m` = messages only)

See the full filter reference at <https://www.aprs-is.net/javAPRSFilter.aspx>.

## Authentication

Port `14580` supports server-side filters but requires a valid passcode to transmit. For receive-only use, `N0CALL` / `-1` is sufficient. Passcodes are derived from your callsign — generate one at <https://apps.magicbug.co.uk/passcode/>.

---

## HTTP server

An optional Axum HTTP server is included, gated behind the `server` feature flag.

### Requirements

- [just](https://github.com/casey/just) — task runner
- A `.env` file in the project root (see below)

### Setup

Copy the example environment and fill in your values:

```
JWT_SECRET=change-me-in-production
DATABASE_URL=sqlite://./dev.db
```

Run migrations (creates the database if it doesn't exist):

```
just migrate
```

### Running

```
just server        # single run
just dev           # hot reload (requires cargo-install cargo-watch)
```

### Configuration

All options fall back to environment variables, then to defaults.

| Flag | Env var | Default | Description |
|------|---------|---------|-------------|
| `--host` | `BIND_ADDR` | `0.0.0.0` | Address to bind on |
| `-p`, `--port` | `PORT` | `3000` | HTTP port |
| `--aprs-server` | `APRS_SERVER` | `rotate.aprs2.net` | APRS-IS hostname |
| `--aprs-port` | `APRS_PORT` | `14580` | APRS-IS port |
| `-u`, `--callsign` | `APRS_CALLSIGN` | `N0CALL` | APRS callsign |
| `--passcode` | `APRS_PASSCODE` | `-1` | APRS passcode |
| `-f`, `--filter` | `APRS_FILTER` | — | Server-side filter |
| `--database-url` | `DATABASE_URL` | `sqlite://./dev.db` | SQLite or Postgres connection URL |
| `--jwt-secret` | `JWT_SECRET` | *(required)* | JWT signing secret |

For Postgres, set `DATABASE_URL=postgres://user:password@host/dbname`.

### API docs

| URL | Description |
|-----|-------------|
| `/swagger-ui` | Swagger UI |
| `/api-docs/openapi.json` | Raw OpenAPI spec |

