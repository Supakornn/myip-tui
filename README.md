# MyIP

<p align="center">
    <img src="/assets/img.png" width="700" alt="MyIP Logo">
</p>

<p align="center" style="font-size: 18px; font-weight: bold;">A clean, intuitive terminal UI for network interface monitoring.</p>

<p align="center">
  <a href="#installation">Installation</a> •
  <a href="#usage">Usage</a> •
  <a href="#interface-details">Interface Details</a> •
<a href="#contributing">Contributing</a>

</p>

## Installation

### Prerequisites

-   Rust toolchain (1.70 or later)
-   Network connectivity (for public IP detection)

### Using Cargo

```bash
# Install the latest version
cargo install myip-tui

# Or specify a version
cargo install myip-tui@0.2.0
```

## Usage

Simply run the application from your terminal:

```bash
myip
```

### Keyboard Controls

| Key   | Action           |
| ----- | ---------------- |
| `q`   | Quit application |
| `ESC` | Quit application |

## Interface Details

MyIP provides a detailed view of your network interfaces in a clean, organized terminal UI:

### Main Screen Elements

1. **Header**: Displays your hostname
2. **Public IP**: Shows your external IP address (fetched from online services)
3. **Interface Panels**: Each network interface is displayed with:
    - Status (up/down)
    - IPv4 and IPv6 addresses
    - MAC address (when available)
    - MTU and link speed (when available)
    - RX/TX traffic statistics
    - Real-time traffic graph
4. **Debug Panel**: Shows detected interfaces and diagnostic information
5. **Footer**: Displays keyboard controls

### Network Traffic Graphs

Each interface panel includes a real-time graph that visualizes:

-   **RX (Download)**: Shown in green
-   **TX (Upload)**: Shown in red

The graph automatically scales based on traffic volume and provides meaningful units (B/s, KB/s, MB/s).

### Public IP Detection

MyIP attempts to fetch your public IP address from multiple services:

-   api.ipify.org
-   ifconfig.me/ip
-   icanhazip.com
-   ipinfo.io/ip
-   myexternalip.com/raw

The application uses a 5-second timeout to ensure responsiveness even if connectivity is limited.

## Troubleshooting

### Common Issues

**No Public IP Displayed**

-   Check your internet connection
-   The application tries multiple services, so one may be blocked
-   Ensure you have the `default-tls` feature enabled in reqwest

**Missing Network Statistics**

-   Some interfaces may not provide statistics through sysinfo
-   The application attempts to fall back to using the `netstat` command
-   Check the debug panel for interfaces with missing statistics

**Network Interfaces Not Showing**

-   Ensure you're running with appropriate permissions
-   Some virtual interfaces or non-standard interfaces may not be detected
-   Check the debug panel to see which interfaces were detected

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

1. Fork the project
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add some amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## License

This project is licensed under the GNU General Public License v3.0 (GPL-3.0) - see the LICENSE file for details.

## Acknowledgements

-   [ratatui](https://github.com/ratatui/ratatui) for the terminal interface library
-   [Crossterm](https://github.com/crossterm-rs/crossterm) for terminal control
-   [Tokio](https://tokio.rs/) for async runtime
-   [Sysinfo](https://github.com/GuillaumeGomez/sysinfo) for system information
-   [local-ip-address](https://github.com/EstebanBorai/local-ip-address) for network interface detection
-   [Reqwest](https://github.com/seanmonstar/reqwest) for HTTP requests

---
