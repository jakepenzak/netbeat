//! Key test reports for netbeat.
//!
//! This module contains the primary reports for netbeat, including the PingReport, SpeedReport, and NetbeatReport.
//! These reports are used to provide detailed information about the network performance of the system after running
//! a speed test against a target server.

use anyhow::Result;
use byte_unit::{Byte, UnitType};
use spinners::{Spinner, Spinners};
use std::{fmt::Display, time::Duration};
use tabled::{
    Table, Tabled,
    settings::{Alignment, Modify, Panel, Remove, Style, object::Rows},
};

/// Utility to print progress information during a speed test.
pub fn print_progress(
    time: Duration,
    bytes: u64,
    spinner: &mut Option<Spinner>,
    preamble: &str,
) -> Option<Spinner> {
    if let Some(spinner) = spinner {
        spinner.stop();
        let speed_megabyte = (bytes as f64 / 1e6) / time.as_secs_f64();
        let unit = Byte::from_u64(bytes).get_appropriate_unit(UnitType::Decimal);
        Some(Spinner::new(
            Spinners::Dots2,
            format!(
                "{preamble} --> Data: {unit:.2} | Speed: {speed_megabyte:.2} MB/s, {:.2} Mbps",
                speed_megabyte * 8.0
            ),
        ))
    } else {
        None
    }
}

/// Represents an individual test metric with an emoji, variable name, pretty name and value.
#[derive(Tabled, Clone)]
pub struct Metric<V: Display> {
    emoji: &'static str,
    #[tabled(skip)]
    var_name: String,
    pretty_name: String,
    value: V,
}

/// Report trait to facilitate common table and json across different tests.
pub trait Report {
    /// Get metrics attribute from underlying test report
    fn get_metrics(&self) -> &[Metric<String>];

    /// Get report title from underlying test report
    fn get_report_title(&self) -> &str;

    /// Convert report to table report
    fn to_table_report(&self) -> impl Display {
        let mut table = Table::new(self.get_metrics());
        table.with((
            Remove::row(Rows::first()),
            Panel::header(self.get_report_title()),
            Style::re_structured_text().remove_top(),
            Modify::new(Rows::first()).with(Alignment::center()),
        ));
        format!("\n{table}\n")
    }

    /// Convert report to json output
    fn to_json(&self) -> impl Display {
        let mut data = json::object![];

        for metric in self.get_metrics() {
            let var_name = metric.var_name.to_string();
            let value = metric.value.to_string();
            data[var_name] = value.into();
        }
        data.dump()
    }
}

/// Primary report for Netbeat, including ping, upload, and download metrics.
pub struct NetbeatReport {
    pub ping_report: PingReport,
    pub upload_report: SpeedReport,
    pub download_report: SpeedReport,
    pub metrics: Vec<Metric<String>>,
}

impl NetbeatReport {
    /// Create a new NetbeatReport instance.
    pub fn new(
        ping_report: PingReport,
        upload_report: SpeedReport,
        download_report: SpeedReport,
    ) -> NetbeatReport {
        let mut metrics = vec![];
        for i in [
            ping_report.get_metrics(),
            upload_report.get_metrics(),
            download_report.get_metrics(),
        ] {
            metrics.extend(i.iter().cloned());
        }

        NetbeatReport {
            ping_report,
            upload_report,
            download_report,
            metrics,
        }
    }
}

impl Report for NetbeatReport {
    fn get_metrics(&self) -> &[Metric<String>] {
        &self.metrics
    }

    fn get_report_title(&self) -> &str {
        "🦀 Netbeat Report"
    }
}

/// Report for ping speed test.
#[derive(Clone)]
pub struct PingReport {
    /// Number of targeted pings
    pub ping_count: u32,
    /// Number of successful pings
    pub successful_pings: u32,
    /// Vector of ping times for each ping
    pub ping_times: Vec<Duration>,
    /// Minimum ping time
    pub min_ping: Duration,
    /// Maximum ping time
    pub max_ping: Duration,
    /// Average ping time
    pub avg_ping: Duration,
    /// Packet loss percentage
    pub packet_loss: f64,
    /// Key Metric objects
    pub metrics: Vec<Metric<String>>,
}

impl PingReport {
    /// Create a new PingReport instance.
    pub fn new(ping_count: u32, successful_pings: u32, ping_times: Vec<Duration>) -> PingReport {
        let min_ping = *ping_times.iter().min().unwrap_or(&Duration::ZERO);
        let max_ping = *ping_times.iter().max().unwrap_or(&Duration::ZERO);
        let avg_ping = if ping_times.is_empty() {
            Duration::ZERO
        } else {
            ping_times.iter().sum::<Duration>() / ping_times.len() as u32
        };
        let packet_loss = (ping_count - successful_pings) as f64 / ping_count as f64 * 100.0;

        let metrics = vec![
            Metric {
                emoji: "📊",
                var_name: "packets_sent".to_string(),
                pretty_name: "Packets sent".to_string(),
                value: ping_count.to_string(),
            },
            Metric {
                emoji: "📈",
                var_name: "packets_received".to_string(),
                pretty_name: "Packets received".to_string(),
                value: successful_pings.to_string(),
            },
            Metric {
                emoji: "📉",
                var_name: "packet_loss".to_string(),
                pretty_name: "Packet loss".to_string(),
                value: format!("{packet_loss:.1}%"),
            },
            Metric {
                emoji: "◾",
                var_name: "min_ping".to_string(),
                pretty_name: "Minimum ping".to_string(),
                value: format!("{min_ping:.2?}"),
            },
            Metric {
                emoji: "⬛",
                var_name: "max_ping".to_string(),
                pretty_name: "Maximum ping".to_string(),
                value: format!("{max_ping:.2?}"),
            },
            Metric {
                emoji: "◼️",
                var_name: "avg_ping".to_string(),
                pretty_name: "Average ping".to_string(),
                value: format!("{avg_ping:.2?}"),
            },
        ];

        PingReport {
            ping_count,
            successful_pings,
            ping_times,
            min_ping,
            max_ping,
            avg_ping,
            packet_loss,
            metrics,
        }
    }
}

impl Report for PingReport {
    fn get_metrics(&self) -> &[Metric<String>] {
        &self.metrics
    }

    fn get_report_title(&self) -> &str {
        "🏓 Ping Report"
    }
}

/// Report for upload/download speed test.
#[derive(Clone)]
pub struct SpeedReport {
    /// Report type (upload/download)
    pub report_type: &'static str,
    /// Elapsed duration of the test
    pub duration: Duration,
    /// Total bytes transferred
    pub bytes: u64,
    /// Speed in bytes per second
    pub speed: f64,
    /// Key Metric Objects
    pub metrics: Vec<Metric<String>>,
}

impl SpeedReport {
    /// Create a new SpeedReport instance.
    pub fn new(report_type: &'static str, duration: Duration, bytes: u64) -> Result<SpeedReport> {
        anyhow::ensure!(
            report_type == "download" || report_type == "upload",
            "Got `{report_type}` expected `download` or `upload`"
        );

        let unit = Byte::from_u64(bytes).get_appropriate_unit(UnitType::Decimal);
        let speed_bytes = (bytes as f64) / (duration.as_secs_f64());
        let speed_megabyte = speed_bytes / (1e6);
        let speed_megabit = speed_megabyte * 8.0;
        let (speed_emoji, speed_metric, byte_metric, elapsed_metric) = match report_type {
            "upload" => ("⏫", "Upload speed", "Uploaded", "Upload time"),
            "download" => ("⏬", "Download speed", "Downloaded", "Download time"),
            _ => unreachable!(),
        };
        let metrics = vec![
            Metric {
                emoji: "📊",
                var_name: byte_metric.to_lowercase(),
                pretty_name: byte_metric.to_string(),
                value: format!("{unit:.2}"),
            },
            Metric {
                emoji: "⏰",
                var_name: elapsed_metric.to_lowercase().replace(" ", "_"),
                pretty_name: elapsed_metric.to_string(),
                value: format!("{duration:.2?}"),
            },
            Metric {
                emoji: speed_emoji,
                var_name: speed_metric.to_lowercase().replace(" ", "_") + "_Mbps",
                pretty_name: format!("{} (Mbps)", speed_metric),
                value: format!("{speed_megabit:.2} Mbps"),
            },
            Metric {
                emoji: speed_emoji,
                var_name: speed_metric.to_lowercase().replace(" ", "_") + "_MBs",
                pretty_name: format!("{} (MB/s)", speed_metric),
                value: format!("{speed_megabyte:.2} MB/s"),
            },
        ];

        Ok(SpeedReport {
            report_type,
            duration,
            bytes,
            speed: bytes as f64 / duration.as_secs_f64(),
            metrics,
        })
    }
}

impl Report for SpeedReport {
    fn get_metrics(&self) -> &[Metric<String>] {
        &self.metrics
    }

    fn get_report_title(&self) -> &str {
        match self.report_type {
            "download" => "⬇️ Download Report",
            "upload" => "⬆️ Upload Report",
            _ => "📊 Speed Report",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_speed_report(report_type: &'static str) -> SpeedReport {
        let time = Duration::from_secs(1);
        let bytes = 1e6 as u64;
        SpeedReport::new(report_type, time, bytes).unwrap()
    }

    fn create_ping_report(no_pings: bool) -> PingReport {
        let ping_count = 4;
        let successful_pings = if no_pings { 0 } else { 2 };
        let ping_times = if no_pings {
            vec![]
        } else {
            vec![
                Duration::from_secs(1),
                Duration::from_secs(2),
                Duration::from_secs(3),
                Duration::from_secs(4),
            ]
        };
        PingReport::new(ping_count, successful_pings, ping_times)
    }

    #[test]
    fn test_upload_speed_report() {
        let report = create_speed_report("upload");
        let metrics = report.get_metrics();
        assert_eq!(metrics.len(), 4);
        assert_eq!(metrics[0].emoji, "📊");
        assert_eq!(metrics[0].var_name, "uploaded");
        assert_eq!(metrics[0].pretty_name, "Uploaded");
        assert_eq!(metrics[0].value, "1.00 MB");
        assert_eq!(metrics[1].emoji, "⏰");
        assert_eq!(metrics[1].var_name, "upload_time");
        assert_eq!(metrics[1].pretty_name, "Upload time");
        assert_eq!(metrics[1].value, "1.00s");
        assert_eq!(metrics[2].emoji, "⏫");
        assert_eq!(metrics[2].var_name, "upload_speed_Mbps");
        assert_eq!(metrics[2].pretty_name, "Upload speed (Mbps)");
        assert_eq!(metrics[2].value, "8.00 Mbps");
        assert_eq!(metrics[3].emoji, "⏫");
        assert_eq!(metrics[3].var_name, "upload_speed_MBs");
        assert_eq!(metrics[3].pretty_name, "Upload speed (MB/s)");
        assert_eq!(metrics[3].value, "1.00 MB/s");
        let report_title = report.get_report_title();
        assert_eq!(report_title, "⬆️ Upload Report");
    }

    #[test]
    fn test_download_speed_report() {
        let report = create_speed_report("download");
        let metrics = report.get_metrics();
        assert_eq!(metrics.len(), 4);
        assert_eq!(metrics[0].emoji, "📊");
        assert_eq!(metrics[0].var_name, "downloaded");
        assert_eq!(metrics[0].pretty_name, "Downloaded");
        assert_eq!(metrics[0].value, "1.00 MB");
        assert_eq!(metrics[1].emoji, "⏰");
        assert_eq!(metrics[1].var_name, "download_time");
        assert_eq!(metrics[1].pretty_name, "Download time");
        assert_eq!(metrics[1].value, "1.00s");
        assert_eq!(metrics[2].emoji, "⏬");
        assert_eq!(metrics[2].var_name, "download_speed_Mbps");
        assert_eq!(metrics[2].pretty_name, "Download speed (Mbps)");
        assert_eq!(metrics[2].value, "8.00 Mbps");
        assert_eq!(metrics[3].emoji, "⏬");
        assert_eq!(metrics[3].var_name, "download_speed_MBs");
        assert_eq!(metrics[3].pretty_name, "Download speed (MB/s)");
        assert_eq!(metrics[3].value, "1.00 MB/s");
        let report_title = report.get_report_title();
        assert_eq!(report_title, "⬇️ Download Report");
    }

    #[test]
    fn test_ping_report() {
        let report = create_ping_report(false);
        let metrics = report.get_metrics();
        assert_eq!(metrics.len(), 6);
        assert_eq!(metrics[0].emoji, "📊");
        assert_eq!(metrics[0].var_name, "packets_sent");
        assert_eq!(metrics[0].pretty_name, "Packets sent");
        assert_eq!(metrics[0].value, "4");
        assert_eq!(metrics[1].emoji, "📈");
        assert_eq!(metrics[1].var_name, "packets_received");
        assert_eq!(metrics[1].pretty_name, "Packets received");
        assert_eq!(metrics[1].value, "2");
        assert_eq!(metrics[2].emoji, "📉");
        assert_eq!(metrics[2].var_name, "packet_loss");
        assert_eq!(metrics[2].pretty_name, "Packet loss");
        assert_eq!(metrics[2].value, "50.0%");
        assert_eq!(metrics[3].emoji, "◾");
        assert_eq!(metrics[3].var_name, "min_ping");
        assert_eq!(metrics[3].pretty_name, "Minimum ping");
        assert_eq!(metrics[3].value, "1.00s");
        assert_eq!(metrics[4].emoji, "⬛");
        assert_eq!(metrics[4].var_name, "max_ping");
        assert_eq!(metrics[4].pretty_name, "Maximum ping");
        assert_eq!(metrics[4].value, "4.00s");
        assert_eq!(metrics[5].emoji, "◼️");
        assert_eq!(metrics[5].var_name, "avg_ping");
        assert_eq!(metrics[5].pretty_name, "Average ping");
        assert_eq!(metrics[5].value, "2.50s");
        let report_title = report.get_report_title();
        assert_eq!(report_title, "🏓 Ping Report");

        // No successful pings
        let report = create_ping_report(true);
        let metrics = report.get_metrics();
        assert_eq!(metrics.len(), 6);
        assert_eq!(metrics[0].emoji, "📊");
        assert_eq!(metrics[0].var_name, "packets_sent");
        assert_eq!(metrics[0].pretty_name, "Packets sent");
        assert_eq!(metrics[0].value, "4");
        assert_eq!(metrics[1].emoji, "📈");
        assert_eq!(metrics[1].var_name, "packets_received");
        assert_eq!(metrics[1].pretty_name, "Packets received");
        assert_eq!(metrics[1].value, "0");
        assert_eq!(metrics[2].emoji, "📉");
        assert_eq!(metrics[2].var_name, "packet_loss");
        assert_eq!(metrics[2].pretty_name, "Packet loss");
        assert_eq!(metrics[2].value, "100.0%");
        assert_eq!(metrics[3].emoji, "◾");
        assert_eq!(metrics[3].var_name, "min_ping");
        assert_eq!(metrics[3].pretty_name, "Minimum ping");
        assert_eq!(metrics[3].value, "0.00ns");
        assert_eq!(metrics[4].emoji, "⬛");
        assert_eq!(metrics[4].var_name, "max_ping");
        assert_eq!(metrics[4].pretty_name, "Maximum ping");
        assert_eq!(metrics[4].value, "0.00ns");
        assert_eq!(metrics[5].emoji, "◼️");
        assert_eq!(metrics[5].var_name, "avg_ping");
        assert_eq!(metrics[5].pretty_name, "Average ping");
        assert_eq!(metrics[5].value, "0.00ns");
    }

    #[test]
    fn test_netbeat_report() {
        let download_report = create_speed_report("download");
        let upload_report = create_speed_report("upload");
        let ping_report = create_ping_report(false);
        let netbeat_report = NetbeatReport::new(
            ping_report.clone(),
            upload_report.clone(),
            download_report.clone(),
        );

        let mut expected_metrics = vec![];
        for i in [
            ping_report.get_metrics(),
            upload_report.get_metrics(),
            download_report.get_metrics(),
        ] {
            expected_metrics.extend(i.iter().cloned());
        }

        let metrics = netbeat_report.get_metrics();

        assert_eq!(metrics.len(), expected_metrics.len());
        for (i, metric) in metrics.iter().enumerate() {
            assert_eq!(metric.emoji, expected_metrics[i].emoji);
            assert_eq!(metric.var_name, expected_metrics[i].var_name);
            assert_eq!(metric.pretty_name, expected_metrics[i].pretty_name);
            assert_eq!(metric.value, expected_metrics[i].value);
        }

        let report_title = netbeat_report.get_report_title();
        assert_eq!(report_title, "🦀 Netbeat Report");
    }

    #[test]
    fn test_report_to_json() {
        let upload_report = create_speed_report("upload");
        let json = upload_report.to_json();

        assert_eq!(
            json.to_string(),
            "{\"uploaded\":\"1.00 MB\",\"upload_time\":\"1.00s\",\"upload_speed_Mbps\":\"8.00 Mbps\",\"upload_speed_MBs\":\"1.00 MB/s\"}"
        );
    }

    #[test]
    fn test_report_to_table_report() {
        let upload_report = create_speed_report("upload");
        let table = upload_report.to_table_report();

        let table_string = table.to_string();

        // The table should not be empty
        assert!(!table_string.is_empty());

        // Should contain the report title
        assert!(table_string.contains("⬆️ Upload Report"));

        // Should contain all the metric data
        assert!(table_string.contains("📊")); // Uploaded emoji
        assert!(table_string.contains("Uploaded"));
        assert!(table_string.contains("1.00 MB"));

        assert!(table_string.contains("⏰")); // Upload time emoji
        assert!(table_string.contains("Upload time"));
        assert!(table_string.contains("1.00s"));

        assert!(table_string.contains("⏫")); // Upload speed emoji
        assert!(table_string.contains("Upload speed (MB/s)"));
        assert!(table_string.contains("1.00 MB/s"));

        assert!(table_string.contains("⏫")); // Upload speed emoji
        assert!(table_string.contains("Upload speed (Mbps)"));
        assert!(table_string.contains("8.00 Mbps"));

        // Should have proper formatting (newlines at start and end)
        assert!(table_string.starts_with('\n'));
        assert!(table_string.ends_with('\n'));
    }

    #[test]
    fn test_print_progress_with_spinner() {
        let time = Duration::from_secs(2);
        let bytes = 2 * 1024 * 1024; // 2 MiB
        let mut spinner = Some(Spinner::new(Spinners::Dots, "Initial message".to_string()));
        let preamble = "Testing";

        let result = print_progress(time, bytes, &mut spinner, preamble);

        // Should return a new spinner
        assert!(result.is_some());

        // Idk how to extract message itself to test contents.
    }

    #[test]
    fn test_print_progress_without_spinner() {
        let time = Duration::from_secs(1);
        let bytes = 1024 * 1024; // 1 MiB
        let mut spinner = None;
        let preamble = "Testing";

        let result = print_progress(time, bytes, &mut spinner, preamble);

        // Should return None when no spinner is provided
        assert!(result.is_none());
    }
}
