pub enum Align {
    Left,
    Center,
    Right,
}

pub struct MarkdownTable {
    pub headers: Vec<String>,
    pub aligns: Vec<Align>,
    pub rows: Vec<Vec<String>>,
}

fn slash_aware_split(s: &str, c: char) -> Vec<String> {
    let mut parts = Vec::new();
    let mut part = String::new();
    let mut escape = false;
    for ch in s.chars() {
        if ch == '\\' {
            escape = true;
        } else if ch == c && !escape {
            parts.push(part);
            part = String::new();
        } else {
            part.push(ch);
            escape = false;
        }
    }
    parts.push(part);
    parts
}

impl MarkdownTable {
    pub fn parse(input: &str) -> Self {
        let mut headers = Vec::new();
        let mut aligns = Vec::new();
        let mut rows = Vec::new();
        let mut lines = input.lines();
        let header_line = lines.next().unwrap();
        let align_line = lines.next().unwrap();
        let header_parts = header_line.split('|').collect::<Vec<_>>();
        let align_parts = align_line.split('|').collect::<Vec<_>>();
        for (i, part) in header_parts.iter().enumerate() {
            if i == 0 || i == header_parts.len() - 1 {
                continue;
            }
            headers.push(part.trim().to_string());
        }
        for (i, part) in align_parts.iter().enumerate() {
            if i == 0 || i == align_parts.len() - 1 {
                continue;
            }
            let part = part.trim();
            let align = if part.starts_with(':') && part.ends_with(':') {
                Align::Center
            } else if part.starts_with(':') {
                Align::Left
            } else if part.ends_with(':') {
                Align::Right
            } else {
                Align::Left
            };
            aligns.push(align);
        }
        for line in lines {
            let parts = slash_aware_split(line, '|');
            let mut row = Vec::new();
            for (i, part) in parts.iter().enumerate() {
                if i == 0 || i == parts.len() - 1 {
                    continue;
                }
                row.push(part.trim().to_string());
            }
            rows.push(row);
        }
        MarkdownTable {
            headers,
            aligns,
            rows,
        }
    }
}

pub fn transform_tables(input: &str) -> String {
    let mut output = String::new();
    enum State {
        Normal,
        Table(String),
        Raw,
    }
    let mut state = State::Normal;
    for line in input.lines() {
        match state {
            State::Normal => {
                if line.trim().starts_with("```") {
                    state = State::Raw;
                    output.push_str(line);
                    output.push_str("\n");
                } else if line.trim().starts_with('|') {
                    state = State::Table(line.to_string());
                } else {
                    output.push_str(line);
                    output.push_str("\n");
                }
            }
            State::Table(ref mut table) => {
                if line.trim().starts_with('|') {
                    table.push_str("\n");
                    table.push_str(line);
                } else {
                    let table = MarkdownTable::parse(&table);
                    output.push_str(&table.to_typst_table());
                    output.push_str("\n");
                    output.push_str(line);
                    output.push_str("\n");
                    state = State::Normal;
                }
            }
            State::Raw => {
                output.push_str(line);
                output.push_str("\n");
                if line.trim().starts_with("```") {
                    state = State::Normal;
                }
            }
        }
    }
    output
}

impl MarkdownTable {
    pub fn to_typst_table(&self) -> String {
        [
            "#table(".to_string(),
            format!(
                "columns: ({}),",
                std::iter::repeat("auto")
                    .take(self.headers.len())
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            format!(
                "align: ({}),",
                self.aligns
                    .iter()
                    .map(|a| match a {
                        Align::Left => "left",
                        Align::Center => "center",
                        Align::Right => "right",
                    })
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            self.headers
                .iter()
                .map(|h| format!("[{}],", h))
                .collect::<Vec<_>>()
                .join(""),
            self.rows
                .iter()
                .flatten()
                .map(|r| format!("[{}],", r))
                .collect::<Vec<_>>()
                .join(""),
            ")".to_string(),
        ]
        .join("\n")
        .to_string()
    }
}
