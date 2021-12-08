const MINUTES: u64 = 60;
const HOURS: u64 = MINUTES * 60;
const DAYS: u64 = HOURS * 24;
const WEEKS: u64 = DAYS * 7;
const MONTHS: u64 = DAYS * 30;
const YEARS: u64 = DAYS * 365;

pub fn parse<S: AsRef<str>>(duration: S) -> Result<u64, String> {
    lazy_static::lazy_static!(
        static ref RE_DURATION: regex::Regex = regex::Regex::new(r"(\d+)(s|m|h|d|w|M|y)").unwrap();
        static ref RE_TIME: regex::Regex = regex::Regex::new(r"(\d{1,2}):(\d{2})(?:(\d{2}))?").unwrap();
    );
    if let Some(dur_captures) = RE_DURATION.captures(duration.as_ref()) {
        let mut duration = dur_captures.get(1).unwrap().as_str().parse::<u64>().unwrap();
        let unit = dur_captures.get(2).unwrap().as_str();
        duration *= match unit {
            "s" => 1,
            "m" => MINUTES,
            "h" => HOURS,
            "d" => DAYS,
            "w" => WEEKS,
            "M" => MONTHS,
            "y" => YEARS,
            _ => return Err(format!(r#"Unité de durée "{}" inconnue, attendue: s, m, h, d, w, M, y"#, unit)),
        };
        Ok(duration)
    } else if let Some(dur_captures) = RE_TIME.captures(duration.as_ref()) {
        let hours = dur_captures.get(1).unwrap().as_str().parse::<u64>().unwrap();
        let minutes = dur_captures.get(2).unwrap().as_str().parse::<u64>().unwrap();
        let seconds = match dur_captures.get(3) {
            Some(s) => s.as_str().parse::<u64>().unwrap(),
            None => 0,
        };
        Ok(hours * HOURS + minutes * MINUTES + seconds)
    } else {
        return Err("Format de la durée invalide".to_string());
    }
}