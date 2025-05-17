# clup

**clup** is a CLI application for monitoring a Patroni PostgreSQL cluster and related service components.

## Features

- **Cluster Overview**: Displays detailed information about the Patroni cluster, including node roles (Primary/Replica), leader node, and replication status.
- **Service Monitoring**: Tracks the status of related services, such as:
  - PgBouncer
  - HAProxy
  - Keepalived
- **VIP Detection**: Automatically detects and displays the Keepalived virtual IP (VIP).
- **Real-Time Stats**: Provides updates on connection counts, replication lag, and backend service health in real-time.
- **Log Inspection**: Allows inspection of cluster and service logs directly within the terminal.

## Installation

1. Clone this repository:
   ```bash
   git clone <repository-url>
   cd clup
   ```

2. Build the application using Rust's `cargo` tool:
   ```bash
   cargo build --release
   ```

3. (Optional) Add the binary to your `PATH`:
   ```bash
   mv target/release/clup /usr/local/bin/
   ```
