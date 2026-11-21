//! Tiny text templates for presence lines, shared by every app.
//!
//!   {name}      inserts a value
//!   [ ... ]     optional part: dropped when any value inside it is missing or hidden
//!
//! e.g. "{script}[ (Line {line} / {lines})]" → "Main (Line 4 / 80)", or just "Main"
//! when line numbers are hidden. The result is tidied (spaces, stray separators) and
//! starts with a capital letter, unless it starts with a name you gave something
//! (a file called "spaceship_v3" stays lower-case).

/// Marks where an inserted value starts, so `tidy` can tell names from template text.
const VALUE_START: char = '\u{E000}';

pub fn render(template: &str, var: &dyn Fn(&str) -> Option<String>) -> String {
    let mut out = String::new();
    // Text of the open optional group, and whether all its values resolved.
    let mut group: Option<(String, bool)> = None;
    let mut chars = template.chars().peekable();

    while let Some(c) = chars.next() {
        match c {
            '[' if group.is_none() => group = Some((String::new(), true)),
            ']' if group.is_some() => {
                let (text, ok) = group.take().unwrap();
                if ok {
                    out.push_str(&text);
                }
            }
            '{' => {
                let mut name = String::new();
                let mut closed = false;
                while let Some(&n) = chars.peek() {
                    chars.next();
                    if n == '}' {
                        closed = true;
                        break;
                    }
                    name.push(n);
                }
                let (text, missing) = if closed {
                    match var(name.trim()).filter(|v| !v.trim().is_empty()) {
                        Some(v) => (format!("{VALUE_START}{v}"), false),
                        None => (String::new(), true),
                    }
                } else {
                    (format!("{{{name}"), false)
                };
                match &mut group {
                    Some((t, ok)) => {
                        t.push_str(&text);
                        if missing {
                            *ok = false;
                        }
                    }
                    None => out.push_str(&text),
                }
            }
            _ => match &mut group {
                Some((t, _)) => t.push(c),
                None => out.push(c),
            },
        }
    }
    if let Some((text, ok)) = group {
        if ok {
            out.push('[');
            out.push_str(&text);
        }
    }
    tidy(&out)
}

fn tidy(s: &str) -> String {
    let mut out = s.split_whitespace().collect::<Vec<_>>().join(" ");
    for empty in ["()", "( )", "[]"] {
        out = out.replace(empty, "");
    }
    let seps: &[char] = &['·', '•', '|', '-', '–', '—', ',', ':', ' ', VALUE_START];
    // Separators before the first value still go, but remember if a value came first.
    let lead = out.trim_start_matches(|c: char| seps.contains(&c) && c != VALUE_START);
    let starts_with_value = lead.starts_with(VALUE_START);
    let out = out
        .trim_matches(seps)
        .replace(VALUE_START, "")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");
    // Template text is capitalised; names stay as written, except our own
    // stand-ins like "a script" / "a private project".
    let stand_in = out.starts_with("a ") || out.starts_with("an ");
    if starts_with_value && !stand_in {
        return out;
    }
    let mut chars = out.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().chain(chars).collect(),
        None => String::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn vars(name: &str) -> Option<String> {
        match name {
            "script" => Some("Main".into()),
            "line" => Some("4".into()),
            "lines" => Some("80".into()),
            "place" => Some("Obby".into()),
            _ => None,
        }
    }

    #[test]
    fn fills_values_and_optional_groups() {
        assert_eq!(render("{script}[ (Line {line} / {lines})]", &vars), "Main (Line 4 / 80)");
        assert_eq!(render("{script}[ (Line {missing} / {lines})]", &vars), "Main");
        assert_eq!(render("[Line {line} of {lines} · ]{place}", &vars), "Line 4 of 80 · Obby");
        assert_eq!(render("[{missing} · ]{place}", &vars), "Obby");
    }

    #[test]
    fn tidies_and_capitalises() {
        assert_eq!(render("  editing   {script} · ", &vars), "Editing Main");
        assert_eq!(render("{missing} · {place}", &vars), "Obby");
        assert_eq!(render("a script ({missing})", &vars), "A script");
        assert_eq!(render("{unclosed", &vars), "{unclosed");
    }

    #[test]
    fn names_keep_their_case() {
        let v = |n: &str| match n {
            "file" => Some("spaceship_v3".to_owned()),
            "hidden" => Some("a private project".to_owned()),
            _ => None,
        };
        assert_eq!(render("[{file}]", &v), "spaceship_v3");
        assert_eq!(render("{missing} · {file}", &v), "spaceship_v3");
        assert_eq!(render("{hidden}", &v), "A private project");
        assert_eq!(render("editing {file}", &v), "Editing spaceship_v3");
    }
}
