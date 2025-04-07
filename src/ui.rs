use crate::network::NetworkInfo;
use ratatui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, Cell, Paragraph, Row, Table, Wrap},
    Frame,
};

pub fn render_ui<B: Backend>(f: &mut Frame<B>, network_info: &NetworkInfo) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(7),
            Constraint::Length(3),
        ])
        .split(f.size());

    render_header(f, chunks[0], network_info);
    
    render_interfaces(f, chunks[1], network_info);
    
    render_debug_info(f, chunks[2], network_info);
    
    render_footer(f, chunks[3]);
}

fn render_header<B: Backend>(f: &mut Frame<B>, area: Rect, network_info: &NetworkInfo) {
    let header_text = vec![Spans::from(vec![
        Span::styled(
            "Network Information for ",
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            &network_info.hostname,
            Style::default().fg(Color::Green).add_modifier(Modifier::BOLD),
        ),
    ])];

    let header = Paragraph::new(header_text)
        .block(Block::default()
            .borders(Borders::ALL)
            .title("MyIP")
            .border_style(Style::default().fg(Color::Blue)))
        .style(Style::default().fg(Color::White))
        .alignment(ratatui::layout::Alignment::Center);

    f.render_widget(header, area);
}

fn render_interfaces<B: Backend>(f: &mut Frame<B>, area: Rect, network_info: &NetworkInfo) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(0),
        ])
        .split(area);
    
    render_public_ip(f, chunks[0], network_info);
    
    let interface_count = network_info.interfaces.len();
    if interface_count == 0 {
        return;
    }
    
    let constraints: Vec<Rect> = if interface_count <= 2 {
        Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![Constraint::Ratio(1, interface_count as u32); interface_count])
            .split(chunks[1])
            .iter()
            .copied()
            .collect()
    } else {
        let rows = interface_count.div_ceil(2);
        let interfaces_area = chunks[1];
        
        let row_constraints = vec![Constraint::Ratio(1, rows as u32); rows];
        let rows_layout: Vec<Rect> = Layout::default()
            .direction(Direction::Vertical)
            .constraints(row_constraints)
            .split(interfaces_area)
            .iter()
            .copied()
            .collect();
        
        let mut interface_areas = Vec::new();
        for row in rows_layout {
            let cols = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Ratio(1, 2), Constraint::Ratio(1, 2)])
                .split(row);
            interface_areas.extend_from_slice(&cols);
        }
        
        interface_areas
    };
    
    for (i, interface) in network_info.interfaces.iter().enumerate() {
        if i < constraints.len() {
            render_interface(f, constraints[i], interface);
        }
    }
}

fn render_public_ip<B: Backend>(f: &mut Frame<B>, area: Rect, network_info: &NetworkInfo) {
    let (public_ip, ip_color) = match &network_info.public_ip {
        Some(ip) => (ip.as_str(), Color::Green),
        None => ("Unknown", Color::Red),
    };
    
    let text = vec![Spans::from(vec![
        Span::styled(
            "Public IP: ",
            Style::default().fg(Color::White).add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            public_ip,
            Style::default().fg(ip_color).add_modifier(Modifier::BOLD),
        ),
    ])];
    
    let paragraph = Paragraph::new(text)
        .block(Block::default()
            .borders(Borders::ALL)
            .title(" External IP ")
            .border_style(Style::default().fg(Color::Magenta)))
        .style(Style::default().fg(Color::White))
        .alignment(ratatui::layout::Alignment::Center);
    
    f.render_widget(paragraph, area);
}

fn render_interface<B: Backend>(f: &mut Frame<B>, area: Rect, interface: &crate::network::Interface) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(8),
            Constraint::Length(7),
        ])
        .split(area);
    
    render_interface_info(f, chunks[0], interface);
    
    render_network_graph(f, chunks[1], interface);
}

fn render_interface_info<B: Backend>(f: &mut Frame<B>, area: Rect, interface: &crate::network::Interface) {
    let mut rows = Vec::new();
    
    let status_color = if interface.status { Color::Green } else { Color::Red };
    let status_text = if interface.status { "up" } else { "down" };
    rows.push(Row::new(vec![
        Cell::from("Status").style(Style::default().fg(Color::Cyan)),
        Cell::from(status_text).style(Style::default().fg(status_color).add_modifier(Modifier::BOLD)),
    ]));
    
    if let Some(mac) = &interface.mac_address {
        rows.push(Row::new(vec![
            Cell::from("MAC Address").style(Style::default().fg(Color::Cyan)),
            Cell::from(mac.as_str()).style(Style::default().fg(Color::Yellow)),
        ]));
    }
    
    if let Some(mtu) = &interface.mtu {
        rows.push(Row::new(vec![
            Cell::from("MTU").style(Style::default().fg(Color::Cyan)),
            Cell::from(mtu.to_string()).style(Style::default().fg(Color::White)),
        ]));
    }
    
    if let Some(speed) = &interface.speed {
        rows.push(Row::new(vec![
            Cell::from("Speed").style(Style::default().fg(Color::Cyan)),
            Cell::from(format!("{} Mbps", speed)).style(Style::default().fg(Color::White)),
        ]));
    }
    
    if interface.received_bytes > 0 {
        rows.push(Row::new(vec![
            Cell::from("RX Bytes").style(Style::default().fg(Color::Cyan)),
            Cell::from(format_bytes(interface.received_bytes)).style(Style::default().fg(Color::Magenta)),
        ]));
    }
    
    if interface.transmitted_bytes > 0 {
        rows.push(Row::new(vec![
            Cell::from("TX Bytes").style(Style::default().fg(Color::Cyan)),
            Cell::from(format_bytes(interface.transmitted_bytes)).style(Style::default().fg(Color::Magenta)),
        ]));
    }
    
    for (i, addr) in interface.ipv4_addresses.iter().enumerate() {
        let prefix = if i == 0 { "IPv4 Address" } else { "" };
        rows.push(Row::new(vec![
            Cell::from(prefix).style(Style::default().fg(Color::Cyan)),
            Cell::from(addr.as_str()).style(Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
        ]));
    }
    
    for (i, addr) in interface.ipv6_addresses.iter().enumerate() {
        let prefix = if i == 0 { "IPv6 Address" } else { "" };
        rows.push(Row::new(vec![
            Cell::from(prefix).style(Style::default().fg(Color::Cyan)),
            Cell::from(addr.as_str()).style(Style::default().fg(Color::Blue)),
        ]));
    }
    
    let table = Table::new(rows)
        .header(Row::new(vec![
            Cell::from("Property").style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Cell::from("Value").style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        ]))
        .block(Block::default()
            .borders(Borders::ALL)
            .title(format!(" {} ", interface.name))
            .border_style(Style::default().fg(Color::Cyan)))
        .widths(&[
            Constraint::Percentage(30),
            Constraint::Percentage(70),
        ])
        .column_spacing(1)
        .highlight_style(Style::default().add_modifier(Modifier::BOLD))
        .style(Style::default().fg(Color::White));
    
    f.render_widget(table, area);
}

fn render_network_graph<B: Backend>(f: &mut Frame<B>, area: Rect, interface: &crate::network::Interface) {
    use ratatui::widgets::{Dataset, Chart, Axis};
    use ratatui::symbols;
    
    let rx_data: Vec<(f64, f64)> = interface.usage.rx_history
        .iter()
        .enumerate()
        .map(|(i, &v)| (i as f64, v))
        .collect();
    
    let tx_data: Vec<(f64, f64)> = interface.usage.tx_history
        .iter()
        .enumerate()
        .map(|(i, &v)| (i as f64, v))
        .collect();

    let max_y = f64::max(interface.usage.max_rx, interface.usage.max_tx) * 1.2;
    
    let (y_max, y_label) = format_rate_for_axis(max_y);
    
    let datasets = vec![
        Dataset::default()
            .name("RX")
            .marker(symbols::Marker::Braille)
            .style(Style::default().fg(Color::Green))
            .data(&rx_data),
        Dataset::default()
            .name("TX")
            .marker(symbols::Marker::Braille)
            .style(Style::default().fg(Color::Red))
            .data(&tx_data),
    ];
    
    let chart = Chart::new(datasets)
        .block(Block::default()
            .title(format!(" Network Traffic ({}) ", y_label))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan)))
        .x_axis(Axis::default()
            .title("Time")
            .style(Style::default().fg(Color::Gray))
            .bounds([0.0, interface.usage.rx_history.len() as f64 - 1.0])
            .labels(vec![]))
        .y_axis(Axis::default()
            .title("Rate")
            .style(Style::default().fg(Color::Gray))
            .bounds([0.0, y_max])
            .labels(vec![
                "0".into(),
                format!("{:.1}", y_max / 4.0).into(),
                format!("{:.1}", y_max / 2.0).into(),
                format!("{:.1}", y_max * 3.0 / 4.0).into(),
                format!("{:.1}", y_max).into(),
            ]));
    
    f.render_widget(chart, area);
}

fn format_rate_for_axis(bytes_per_sec: f64) -> (f64, String) {
    const KB: f64 = 1024.0;
    const MB: f64 = KB * 1024.0;
    
    if bytes_per_sec < KB {
        (bytes_per_sec, "B/s".to_string())
    } else if bytes_per_sec < MB {
        (bytes_per_sec / KB, "KB/s".to_string())
    } else {
        (bytes_per_sec / MB, "MB/s".to_string())
    }
}

fn render_debug_info<B: Backend>(f: &mut Frame<B>, area: Rect, network_info: &NetworkInfo) {
    let mut text = Vec::new();
    
    text.push(Spans::from(vec![
        Span::styled("Debug Information", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
    ]));
    text.push(Spans::from(vec![Span::raw("")]));
    
    if let Some(netif_names) = network_info.debug_info.get("netifs") {
        let joined = netif_names.iter()
            .map(|s| s.to_string())
            .collect::<Vec<_>>()
            .join(", ");
        text.push(Spans::from(vec![
            Span::styled("Network Interfaces: ", Style::default().fg(Color::Blue).add_modifier(Modifier::BOLD)),
            Span::styled(joined, Style::default().fg(Color::Green)),
        ]));
    }
    
    if let Some(sysinfo_names) = network_info.debug_info.get("sysinfo") {
        let joined = sysinfo_names.iter()
            .map(|s| s.to_string())
            .collect::<Vec<_>>()
            .join(", ");
        text.push(Spans::from(vec![
            Span::styled("System Info: ", Style::default().fg(Color::Blue).add_modifier(Modifier::BOLD)),
            Span::styled(joined, Style::default().fg(Color::Cyan)),
        ]));
    }
    
    if let Some(no_stats) = network_info.debug_info.get("interfaces_with_no_stats") {
        if !no_stats.is_empty() {
            let joined = no_stats.iter()
                .map(|s| s.to_string())
                .collect::<Vec<_>>()
                .join(", ");
            text.push(Spans::from(vec![
                Span::styled("No Stats: ", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
                Span::styled(joined, Style::default().fg(Color::White)),
            ]));
        }
    }
    
    let paragraph = Paragraph::new(text)
        .block(Block::default()
            .borders(Borders::ALL)
            .title("Debug Info")
            .border_style(Style::default().fg(Color::Magenta)))
        .style(Style::default().fg(Color::White))
        .wrap(Wrap { trim: true });
    
    f.render_widget(paragraph, area);
}

fn render_footer<B: Backend>(f: &mut Frame<B>, area: Rect) {
    let text = vec![Spans::from(vec![
        Span::styled(
            "Press 'q' or ESC to exit",
            Style::default().fg(Color::White),
        ),
    ])];
    
    let paragraph = Paragraph::new(text)
        .block(Block::default().borders(Borders::ALL))
        .style(Style::default().fg(Color::White))
        .alignment(ratatui::layout::Alignment::Center);
    
    f.render_widget(paragraph, area);
}

fn format_bytes(bytes: u64) -> String {
    const KB: f64 = 1024.0;
    const MB: f64 = KB * 1024.0;
    const GB: f64 = MB * 1024.0;
    
    let bytes_f64 = bytes as f64;
    
    if bytes_f64 < KB {
        format!("{} B", bytes)
    } else if bytes_f64 < MB {
        format!("{:.2} KB", bytes_f64 / KB)
    } else if bytes_f64 < GB {
        format!("{:.2} MB", bytes_f64 / MB)
    } else {
        format!("{:.2} GB", bytes_f64 / GB)
    }
}
