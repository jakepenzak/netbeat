use byte_unit::{Byte, UnitType};
use spinners::{Spinner, Spinners};
use std::time::Duration;

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
