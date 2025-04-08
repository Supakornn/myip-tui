use anyhow::Result;
use local_ip_address::list_afinet_netifas;
use sysinfo::{NetworkExt, NetworksExt, System, SystemExt};
use std::collections::{HashMap, VecDeque};
use std::process::Command;
use tokio::time::timeout;
use std::time::Duration;

const HISTORY_SIZE: usize = 60;

#[derive(Debug, Clone)]
pub struct NetworkUsage {
    pub rx_history: VecDeque<f64>,
    pub tx_history: VecDeque<f64>,
    pub max_rx: f64,
    pub max_tx: f64,
    #[allow(dead_code)]
    pub last_rx: u64,
    #[allow(dead_code)]
    pub last_tx: u64,
}

impl NetworkUsage {
    pub fn new() -> Self {
        let mut rx_history = VecDeque::with_capacity(HISTORY_SIZE);
        let mut tx_history = VecDeque::with_capacity(HISTORY_SIZE);
        
        for _ in 0..HISTORY_SIZE {
            rx_history.push_back(0.0);
            tx_history.push_back(0.0);
        }
        
        NetworkUsage {
            rx_history,
            tx_history,
            max_rx: 1.0,
            max_tx: 1.0,
            last_rx: 0,
            last_tx: 0,
        }
    }
    
    #[allow(dead_code)]
    pub fn update(&mut self, rx_bytes: u64, tx_bytes: u64) {
        let rx_diff = if rx_bytes >= self.last_rx {
            (rx_bytes - self.last_rx) as f64
        } else {
            rx_bytes as f64
        };
        
        let tx_diff = if tx_bytes >= self.last_tx {
            (tx_bytes - self.last_tx) as f64
        } else {
            tx_bytes as f64
        };
        
        self.last_rx = rx_bytes;
        self.last_tx = tx_bytes;
        
        if self.rx_history.len() >= HISTORY_SIZE {
            self.rx_history.pop_front();
        }
        if self.tx_history.len() >= HISTORY_SIZE {
            self.tx_history.pop_front();
        }
        
        self.rx_history.push_back(rx_diff);
        self.tx_history.push_back(tx_diff);
        
        self.max_rx = self.rx_history.iter().copied().fold(1.0, f64::max);
        self.max_tx = self.tx_history.iter().copied().fold(1.0, f64::max);
    }
}

#[derive(Debug, Clone)]
pub struct Interface {
    pub name: String,
    pub ipv4_addresses: Vec<String>,
    pub ipv6_addresses: Vec<String>,
    pub mac_address: Option<String>,
    pub status: bool,
    pub speed: Option<u64>,
    pub mtu: Option<u32>,
    pub received_bytes: u64,
    pub transmitted_bytes: u64,
    pub usage: NetworkUsage,
}

#[derive(Debug, Clone)]
pub struct NetworkInfo {
    pub interfaces: Vec<Interface>,
    pub hostname: String,
    pub public_ip: Option<String>,
    pub debug_info: HashMap<String, Vec<String>>,
    #[allow(dead_code)]
    pub update_count: u64,
}

pub async fn get_network_info() -> Result<NetworkInfo> {
    let hostname = gethostname::gethostname()
        .to_string_lossy()
        .to_string();

    let netifs = list_afinet_netifas()?;
    
    let mut system = System::new_all();
    system.refresh_all();
    let networks = system.networks();
    
    let mut debug_info = HashMap::new();
    debug_info.insert("netifs".to_string(), netifs.iter().map(|(name, _)| name.clone()).collect());
    debug_info.insert("sysinfo".to_string(), networks.iter().map(|(name, _)| name.clone()).collect());
    
    let mut interfaces = Vec::new();
    for (name, ip) in netifs {
        if name.starts_with("lo") || name.to_lowercase().contains("loopback") {
            continue;
        }
        
        let mut ipv4_addresses = Vec::new();
        let mut ipv6_addresses = Vec::new();
        
        match ip {
            std::net::IpAddr::V4(addr) => {
                ipv4_addresses.push(addr.to_string());
            }
            std::net::IpAddr::V6(addr) => {
                ipv6_addresses.push(addr.to_string());
            }
        }
        
        let mut rx_bytes = 0;
        let mut tx_bytes = 0;
        
        let name_lower = name.to_lowercase();
        for (net_name, stats) in networks.iter() {
            if net_name == &name || net_name.to_lowercase() == name_lower {
                rx_bytes = stats.received();
                tx_bytes = stats.transmitted();
                break;
            }
        }
        
        if rx_bytes == 0 && tx_bytes == 0 {
            let base_name = name.chars()
                .skip_while(|c| c.is_alphabetic())
                .collect::<String>();
                
            if !base_name.is_empty() {
                for (net_name, stats) in networks.iter() {
                    let net_base = net_name.chars()
                        .skip_while(|c| c.is_alphabetic())
                        .collect::<String>();
                        
                    if !net_base.is_empty() && net_base == base_name {
                        rx_bytes = stats.received();
                        tx_bytes = stats.transmitted();
                        break;
                    }
                }
            }
        }
        
        let interface = Interface {
            name,
            ipv4_addresses,
            ipv6_addresses,
            mac_address: None,
            status: true,
            speed: None,
            mtu: None,
            received_bytes: rx_bytes,
            transmitted_bytes: tx_bytes,
            usage: NetworkUsage::new(),
        };
        
        interfaces.push(interface);
    }
    
    let mut interfaces_with_no_stats = Vec::new();
    for interface in &interfaces {
        if interface.received_bytes == 0 && interface.transmitted_bytes == 0 {
            interfaces_with_no_stats.push(interface.name.clone());
        }
    }
    
    if !interfaces_with_no_stats.is_empty() {
        debug_info.insert("interfaces_with_no_stats".to_string(), interfaces_with_no_stats.clone());
        
        #[cfg(any(target_os = "linux", target_os = "macos"))]
        {
            if let Ok(output) = Command::new("netstat").arg("-i").output() {
                if let Ok(output_str) = String::from_utf8(output.stdout) {
                    debug_info.insert("netstat_output".to_string(), vec![output_str.clone()]);
                    
                    for line in output_str.lines() {
                        let parts: Vec<&str> = line.split_whitespace().collect();
                        if parts.len() >= 5 {
                            let if_name = parts[0];
                            
                            for interface in &mut interfaces {
                                if interface.name == if_name || interface.name.starts_with(if_name) {
                                    if let Ok(rx) = parts[4].parse::<u64>() {
                                        interface.received_bytes = rx * 1024;
                                    }
                                    if parts.len() >= 8 {
                                        if let Ok(tx) = parts[7].parse::<u64>() {
                                            interface.transmitted_bytes = tx * 1024;
                                        }
                                    }
                                    break;
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    
    let public_ip = (tokio::spawn(get_real_public_ip()).await).unwrap_or_default();

    
    Ok(NetworkInfo {
        interfaces,
        hostname,
        public_ip,
        debug_info,
        update_count: 0,
    })
}

async fn get_real_public_ip() -> Option<String> {
    (timeout(Duration::from_secs(5), async {
        let client = match reqwest::Client::builder()
            .timeout(Duration::from_secs(4))
            .build() {
                Ok(client) => client,
                Err(_) => return None,
            };
            
        for url in [
            "https://api.ipify.org", 
            "https://ifconfig.me/ip", 
            "https://icanhazip.com",
            "https://ipinfo.io/ip",
            "https://myexternalip.com/raw"
        ] {
            match client.get(url).send().await {
                Ok(resp) => {
                    if resp.status().is_success() {
                        if let Ok(text) = resp.text().await {
                            let ip = text.trim().to_string();
                            if !ip.is_empty() {
                                return Some(ip);
                            }
                        }
                    }
                },
                Err(_) => continue,
            }
        }
        None
    }).await).unwrap_or_default()
}

#[allow(dead_code)]
pub async fn update_network_info(info: &mut NetworkInfo) -> Result<()> {
    let mut system = System::new_all();
    system.refresh_all();
    let networks = system.networks();
    
    for interface in &mut info.interfaces {
        let mut rx_bytes = 0;
        let mut tx_bytes = 0;
        
        for (net_name, stats) in networks.iter() {
            if net_name == &interface.name || net_name.to_lowercase() == interface.name.to_lowercase() {
                rx_bytes = stats.received();
                tx_bytes = stats.transmitted();
                break;
            }
        }
        
        if rx_bytes == 0 && tx_bytes == 0 {
            let name = &interface.name;
            let base_name = name.chars()
                .skip_while(|c| c.is_alphabetic())
                .collect::<String>();
                
            if !base_name.is_empty() {
                for (net_name, stats) in networks.iter() {
                    let net_base = net_name.chars()
                        .skip_while(|c| c.is_alphabetic())
                        .collect::<String>();
                        
                    if !net_base.is_empty() && net_base == base_name {
                        rx_bytes = stats.received();
                        tx_bytes = stats.transmitted();
                        break;
                    }
                }
            }
        }
        
        interface.received_bytes = rx_bytes;
        interface.transmitted_bytes = tx_bytes;
        
        interface.usage.update(rx_bytes, tx_bytes);
    }
    
    info.update_count += 1;
    
    Ok(())
}

