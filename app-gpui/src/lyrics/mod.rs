#[derive(Clone, Debug)]
pub struct LyricLine {
    pub time: f64,
    pub content: String,
}

fn parse_tag_time(inner: &str) -> Option<f64> {
    let mut parts = inner.split(':');
    let mm: i32 = parts.next()?.parse().ok()?;
    let rest = parts.next()?;
    let (ss, xx) = if let Some((a, b)) = rest.split_once('.') {
        (a.parse::<i32>().ok()?, b.parse::<i32>().unwrap_or(0))
    } else {
        (rest.parse::<i32>().ok()?, 0)
    };
    Some(f64::from(mm * 60 + ss) + f64::from(xx) / 100.0)
}

fn strip_tags(line: &str) -> String {
    let mut r = line.to_string();
    loop {
        let Some(a) = r.find('[') else { break };
        let Some(b) = r[a..].find(']') else { break };
        let inner = &r[a + 1..a + b];
        if parse_tag_time(inner).is_some() {
            r.replace_range(a..a + b + 1, "");
        } else {
            break;
        }
    }
    r.trim().to_string()
}

fn tag_times_in_line(line: &str) -> Vec<f64> {
    let mut times = Vec::new();
    let mut i = 0usize;
    while i < line.len() {
        let Some(rel) = line[i..].find('[') else { break };
        let s = i + rel;
        let Some(rel2) = line[s + 1..].find(']') else { break };
        let inner = &line[s + 1..s + 1 + rel2];
        if let Some(t) = parse_tag_time(inner) {
            times.push(t);
        }
        i = s + 1 + rel2 + 1;
    }
    times
}

pub fn parse_lrc_text(raw: &str) -> Vec<LyricLine> {
    let mut out = Vec::new();
    for line in raw.lines() {
        let content = strip_tags(line);
        if content.is_empty() {
            continue;
        }
        for time in tag_times_in_line(line) {
            out.push(LyricLine {
                time,
                content: content.clone(),
            });
        }
    }
    out
}

pub fn lyric_index_at_time(lines: &[LyricLine], time: f64) -> Option<usize> {
    if lines.is_empty() {
        return None;
    }
    let mut idx = None;
    for i in 0..lines.len() {
        let cur = lines[i].time;
        let next = lines.get(i + 1).map(|l| l.time);
        match next {
            Some(n) if time >= cur && time < n => {
                idx = Some(i);
                break;
            }
            None if time >= cur => {
                idx = Some(i);
                break;
            }
            _ => {}
        }
    }
    idx
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_lrc_basic() {
        let lines = parse_lrc_text("[00:01.00]hello\n[00:02.00]world");
        assert_eq!(lines.len(), 2);
        assert!((lines[0].time - 1.0).abs() < f64::EPSILON);
        assert_eq!(lines[0].content, "hello");
    }

    #[test]
    fn lyric_index_at_time_picks_active_line() {
        let lines = parse_lrc_text("[00:00.00]a\n[00:05.00]b\n[00:10.00]c");
        assert_eq!(lyric_index_at_time(&lines, 5.5), Some(1));
    }
}
