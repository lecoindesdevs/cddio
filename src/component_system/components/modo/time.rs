const MINUTES: u64 = 60;
const HOURS: u64 = MINUTES * 60;
const DAYS: u64 = HOURS * 24;
const WEEKS: u64 = DAYS * 7;
const MONTHS: u64 = DAYS * 30;
const YEARS: u64 = DAYS * 365;

struct Unit {
    long_name: &'static str,
    short_name: &'static str,
    value: u64,
}
const UNITS: &[Unit] = &[
    Unit {
        long_name: "seconde",
        short_name: "sec",
        value: 1,
    },
    Unit {
        long_name: "minute",
        short_name: "min",
        value: MINUTES,
    },
    Unit {
        long_name: "heure",
        short_name: "h",
        value: HOURS,
    },
    Unit {
        long_name: "jour",
        short_name: "j",
        value: DAYS,
    },
    Unit {
        long_name: "semaine",
        short_name: "sem",
        value: WEEKS,
    },
    Unit {
        long_name: "mois",
        short_name: "mois",
        value: MONTHS,
    },
    Unit {
        long_name: "an",
        short_name: "an",
        value: YEARS,
    },
];


pub fn parse<S: AsRef<str>>(duration: S) -> Result<u64, String> {
    lazy_static::lazy_static!(
        static ref STR_RE_UNITS: String = format!("{}{}", UNITS[0].short_name, UNITS.iter().skip(1).map(|v| format!("|{}",v.short_name)).collect::<String>());
        static ref STR_LIST_UNITS: String = format!("{}{}", UNITS[0].short_name, UNITS.iter().skip(1).map(|v| format!(", {}",v.short_name)).collect::<String>());
        static ref STR_RE_DURATION: String = format!(r"(\d+)({})", *STR_RE_UNITS);
        static ref RE_DURATION: regex::Regex = regex::Regex::new(STR_RE_DURATION.as_str()).unwrap();
        static ref RE_TIME: regex::Regex = regex::Regex::new(r"(\d{1,2}):(\d{2})(?:(\d{2}))?").unwrap();
    );
    if let Some(dur_captures) = RE_DURATION.captures(duration.as_ref()) {
        let mut duration = dur_captures.get(1).unwrap().as_str().parse::<u64>().unwrap();
        let unit = dur_captures.get(2).unwrap().as_str();
        duration *= UNITS.iter()
            .find(|v| v.short_name == unit).map(|v| v.value)
            .ok_or_else(|| format!(r#"Unité de durée "{}" inconnue, attendue: {}"#, unit, *STR_LIST_UNITS))?;
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
        Err(format!("Format de la durée invalide\nMettez un nombre suivi de l'unité.\nListe des unités : {}", *STR_LIST_UNITS))
    }
}

pub fn format_duration(mut duration: u64) -> String {
    let mut result = Vec::new();
    for unit in UNITS.iter().rev() {
        if duration >= unit.value {
            let value = duration / unit.value;
            duration %= unit.value;
            result.push(format!("{} {}{}", value, unit.long_name, if value > 1 && unit.short_name != "mois" { "s" } else { "" }));
        }
    }
    result.join(" ")
}