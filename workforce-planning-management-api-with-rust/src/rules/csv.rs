//! A small, dependency-free **RFC 4180 CSV parser** for importing
//! classification downloads (ESCO's CSV fields hold quoted commas and
//! newlines). Pure: text in, rows out. Handles a UTF-8 byte-order mark,
//! `\r\n` and `\n`, quoted fields, doubled quotes, and blank trailing lines.

/// Parse CSV text into rows of fields. A row with a single empty field (a
/// blank line) is dropped.
///
/// # Errors
/// A message when a quoted field is never closed.
pub fn parse_csv(text: &str) -> Result<Vec<Vec<String>>, String> {
    let text = text.strip_prefix('\u{feff}').unwrap_or(text);
    let mut rows: Vec<Vec<String>> = Vec::new();
    let mut row: Vec<String> = Vec::new();
    let mut field = String::new();
    let mut in_quotes = false;
    let mut chars = text.chars().peekable();
    while let Some(c) = chars.next() {
        if in_quotes {
            match c {
                '"' if chars.peek() == Some(&'"') => {
                    field.push('"');
                    chars.next();
                }
                '"' => in_quotes = false,
                other => field.push(other),
            }
        } else {
            match c {
                '"' if field.is_empty() => in_quotes = true,
                ',' => row.push(std::mem::take(&mut field)),
                '\r' => {}
                '\n' => {
                    row.push(std::mem::take(&mut field));
                    push_row(&mut rows, std::mem::take(&mut row));
                }
                other => field.push(other),
            }
        }
    }
    if in_quotes {
        return Err("a quoted field is never closed".to_string());
    }
    if !field.is_empty() || !row.is_empty() {
        row.push(field);
        push_row(&mut rows, row);
    }
    Ok(rows)
}

fn push_row(rows: &mut Vec<Vec<String>>, row: Vec<String>) {
    if !(row.len() == 1 && row[0].is_empty()) {
        rows.push(row);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plain_rows() {
        let rows = parse_csv("a,b,c\n1,2,3\n").unwrap();
        assert_eq!(rows, [["a", "b", "c"], ["1", "2", "3"]]);
    }

    #[test]
    fn quoted_commas_quotes_and_newlines() {
        let text =
            "uri,label,description\r\nx,\"Dev, senior\",\"Line one\nline \"\"two\"\"\"\r\ny,z,\r\n";
        let rows = parse_csv(text).unwrap();
        assert_eq!(rows.len(), 3);
        assert_eq!(rows[1][1], "Dev, senior");
        assert_eq!(rows[1][2], "Line one\nline \"two\"");
        assert_eq!(rows[2], ["y", "z", ""], "a trailing empty field is kept");
    }

    #[test]
    fn bom_blank_lines_and_missing_final_newline() {
        let rows = parse_csv("\u{feff}a,b\n\n1,2").unwrap();
        assert_eq!(rows, [["a", "b"], ["1", "2"]]);
        assert_eq!(parse_csv("").unwrap(), Vec::<Vec<String>>::new());
    }

    #[test]
    fn unterminated_quote_is_an_error() {
        assert!(parse_csv("a,\"b\n").is_err());
    }
}
