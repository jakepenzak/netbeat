use anyhow::{Result, ensure};
use byte_unit::{Byte, UnitType};
use spinners::{Spinner, Spinners};
use std::{fmt::Display, time::Duration};
use tabled::{
    Table, Tabled,
    settings::{Alignment, Modify, Panel, Remove, Style, object::Rows},
};

pub fn print_progress(
    time: Duration,
    bytes: u64,
    spinner: &mut Spinner,
    preamble: &str,
) -> Spinner {
    spinner.stop();
    let speed_megabyte = (bytes as f64 / 1e6) / time.as_secs_f64();
    let unit = Byte::from_u64(bytes).get_appropriate_unit(UnitType::Binary);
    Spinner::new(
        Spinners::Dots2,
        format!(
            "{preamble} --> Data: {unit:.2} | Speed: {speed_megabyte:.2} MiB/s, {:.2} Mib/s",
            speed_megabyte * 8.0
        ),
    )
}

#[derive(Tabled, Clone)]
pub struct Metric<V: Display> {
    emoji: &'static str,
    desc: &'static str,
    value: V,
}

pub trait Report {
    fn get_metrics(&self) -> &[Metric<String>];
    fn get_report_title(&self) -> &str;

    fn table_report(&self) -> String {
        let mut table = Table::new(self.get_metrics());
        table.with((
            Remove::row(Rows::first()),
            Panel::header(self.get_report_title()),
            Style::re_structured_text().remove_top(),
            Modify::new(Rows::first()).with(Alignment::center()),
        ));
        format!("\n{table}\n")
    }

    fn to_json(&self) -> String {
        let mut data = json::object![];

        for metric in self.get_metrics() {
            let desc = metric.desc.to_string();
            let value = metric.value.to_string();
            data[desc] = value.into();
        }
        data.dump()
    }
}

pub struct NetbeatReport {
    pub ping_report: PingReport,
    pub upload_report: SpeedReport,
    pub download_report: SpeedReport,
    pub metrics: Vec<Metric<String>>,
}

impl NetbeatReport {
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

pub struct PingReport {
    pub report_type: String,
    pub ping_count: u32,
    pub succesful_pings: u32,
    pub ping_times: Vec<Duration>,
    pub min_ping: Duration,
    pub max_ping: Duration,
    pub avg_ping: Duration,
    pub packet_loss: f64,
    pub metrics: Vec<Metric<String>>,
}

impl PingReport {
    pub fn new(ping_count: u32, succesful_pings: u32, ping_times: Vec<Duration>) -> PingReport {
        let min_ping = *ping_times.iter().min().unwrap();
        let max_ping = *ping_times.iter().max().unwrap();
        let avg_ping = ping_times.iter().sum::<Duration>() / ping_times.len() as u32;
        let packet_loss = (ping_count - succesful_pings) as f64 / ping_count as f64 * 100.0;

        let metrics = vec![
            Metric {
                emoji: "📊",
                desc: "Packets sent",
                value: ping_count.to_string(),
            },
            Metric {
                emoji: "📈",
                desc: "Packets received",
                value: succesful_pings.to_string(),
            },
            Metric {
                emoji: "📉",
                desc: "Packet loss",
                value: format!("{packet_loss:.1}%"),
            },
            Metric {
                emoji: "◾",
                desc: "Minimum ping",
                value: format!("{min_ping:.2?}"),
            },
            Metric {
                emoji: "⬛",
                desc: "Maximum ping",
                value: format!("{max_ping:.2?}"),
            },
            Metric {
                emoji: "◼️",
                desc: "Average ping",
                value: format!(" {avg_ping:.2?}"),
            },
        ];

        PingReport {
            report_type: "ping".to_string(),
            ping_count,
            succesful_pings,
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

pub struct SpeedReport {
    pub report_type: &'static str,
    pub duration: Duration,
    pub bytes: u64,
    pub speed: f64,
    pub metrics: Vec<Metric<String>>,
}

impl SpeedReport {
    pub fn new(
        report_type: &'static str,
        duration: Duration,
        bytes: u64,
    ) -> Result<Self, anyhow::Error> {
        ensure!(
            report_type == "download" || report_type == "upload",
            "Got `{report_type}` expected `download` or `upload`"
        );

        let unit = Byte::from_u64(bytes).get_appropriate_unit(UnitType::Binary);
        let speed_bytes = (bytes as f64) / (duration.as_secs_f64());
        let speed_megabyte = speed_bytes / 1e6;
        let speed_megabit = speed_megabyte * 8.0;
        let (speed_emoji, speed_metric, byte_metric, elapsed_metric) = match report_type {
            "upload" => ("⏫", "Upload speed", "Uploaded", "Upload time"),
            "download" => ("⏬", "Download speed", "Downloaded", "Download time"),
            _ => unreachable!(),
        };
        let metrics = vec![
            Metric {
                emoji: "📊",
                desc: byte_metric,
                value: format!("{unit:.2}"),
            },
            Metric {
                emoji: "⏰",
                desc: elapsed_metric,
                value: format!("{duration:.2?}"),
            },
            Metric {
                emoji: speed_emoji,
                desc: speed_metric,
                value: format!("{speed_megabyte:.2} MiB/s, {speed_megabit:.2} Mib/s"),
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
