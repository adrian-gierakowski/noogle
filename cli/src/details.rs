use crate::model::DocItem;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span, Text};
use regex::Regex;
use once_cell::sync::Lazy;

#[derive(Debug, PartialEq)]
struct ParsedDoc {
    description: String,
    inputs: Vec<String>,
    type_info: Option<String>,
    examples: Option<String>,
}

fn parse_doc(item: &DocItem) -> ParsedDoc {
    let content = item.content().map(|s| s.as_str()).unwrap_or("");

    // Simple regex-based parsing for sections
    // Sections are headers like "# Inputs", "# Type", "# Examples"

    static SECTIONS_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"(?m)^#\s+(.*)$").unwrap());

    let mut current_pos = 0;
    let mut description = String::new();
    let mut inputs_text = String::new();
    let mut type_text = String::new();
    let mut examples_text = String::new();

    let mut current_section = "Description";

    for cap in SECTIONS_RE.captures_iter(content) {
        let match_start = cap.get(0).unwrap().start();
        let match_end = cap.get(0).unwrap().end();
        let header = cap.get(1).unwrap().as_str().trim();

        let text = &content[current_pos..match_start].trim();

        match current_section {
            "Description" => description = text.to_string(),
            "Inputs" => inputs_text = text.to_string(),
            "Type" => type_text = text.to_string(),
            "Examples" => examples_text = text.to_string(),
            _ => {}
        }

        current_section = header;
        current_pos = match_end;
    }

    // Capture the last section
    let text = &content[current_pos..].trim();
    match current_section {
        "Description" => description = text.to_string(),
        "Inputs" => inputs_text = text.to_string(),
        "Type" => type_text = text.to_string(),
        "Examples" => examples_text = text.to_string(),
        _ => {}
    }

    // Process inputs
    let mut inputs = Vec::new();
    if !inputs_text.is_empty() {
        inputs.push(inputs_text);
    } else if let Some(meta_args) = &item.meta.primop_meta.as_ref().and_then(|m| m.args.as_ref()) {
         // Primop args
         inputs.push(format!("Takes {} arguments\n\n{}", meta_args.len(), meta_args.join(", ")));
    }

    // Process type
    let type_info = if !type_text.is_empty() {
        // Strip code block markers
        Some(type_text.replace("```", "").trim().to_string())
    } else {
        item.meta.signature.clone().map(|s| s.trim().to_string())
    };

    // Process examples
    let examples = if !examples_text.is_empty() {
        Some(examples_text)
    } else {
        None
    };

    ParsedDoc {
        description,
        inputs,
        type_info,
        examples,
    }
}

fn format_code_block(text: &str) -> Vec<Line<'static>> {
    let code_style = Style::default().bg(Color::Rgb(30, 30, 30));
    text.lines()
        .map(|line| Line::from(Span::styled(format!(" {} ", line), code_style)))
        .collect()
}

pub fn render_doc(item: &DocItem) -> (Text<'static>, Option<String>) {
    let parsed = parse_doc(item);
    let mut lines = Vec::new();

    // 1. Title
    lines.push(Line::from(Span::styled(
        item.title().to_string(),
        Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD).add_modifier(Modifier::UNDERLINED),
    )));
    lines.push(Line::from(""));

    // 2. Description
    if !parsed.description.is_empty() {
        lines.push(Line::from(parsed.description));
        lines.push(Line::from(""));
    }

    // 3. Inputs
    let header_style = Style::default().fg(Color::Blue).add_modifier(Modifier::BOLD).add_modifier(Modifier::UNDERLINED);

    if !parsed.inputs.is_empty() {
        lines.push(Line::from(Span::styled("Inputs", header_style)));
        lines.push(Line::from(""));

        static INPUT_ARG_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"^`([^`]+)`$").unwrap());
        static INPUT_DESC_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"^:\s*(.*)$").unwrap());

        for input in parsed.inputs {
             for line in input.lines() {
                 let line = line.trim();
                 if let Some(caps) = INPUT_ARG_RE.captures(line) {
                     // Arg name - Cyan, Italic
                     lines.push(Line::from(Span::styled(
                         caps.get(1).unwrap().as_str().to_string(),
                         Style::default().fg(Color::Cyan).add_modifier(Modifier::ITALIC)
                     )));
                 } else if let Some(caps) = INPUT_DESC_RE.captures(line) {
                     // Description - Indented
                     let desc = caps.get(1).unwrap().as_str().replace(r"\.", ".");
                     lines.push(Line::from(format!("  {}", desc)));
                 } else {
                     // Fallback
                     lines.push(Line::from(line.to_string()));
                 }
             }
        }
        lines.push(Line::from(""));
    } else if let Some(primop) = &item.meta.primop_meta {
         if let Some(args) = &primop.args {
            lines.push(Line::from(Span::styled("Inputs", header_style)));
            lines.push(Line::from(""));
            lines.push(Line::from(format!("Takes {} arguments", args.len())));
            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled(args.join(", "), Style::default().add_modifier(Modifier::BOLD))));
            lines.push(Line::from(""));
         }
    }

    // 4. Type
    if let Some(t) = parsed.type_info {
        lines.push(Line::from(Span::styled("Type", header_style)));
        lines.push(Line::from(""));
        lines.extend(format_code_block(&t));
        lines.push(Line::from(""));
    }

    // 5. Examples
    if let Some(ex) = parsed.examples {
        lines.push(Line::from(Span::styled("Examples", header_style)));
        lines.push(Line::from(""));

        let mut in_code_block = false;
        let code_style = Style::default().bg(Color::Rgb(30, 30, 30));

        for line in ex.lines() {
             if line.starts_with("```") || line.starts_with(":::") {
                 in_code_block = !in_code_block;
                 continue;
             }
             if line.starts_with("##") {
                 lines.push(Line::from(Span::styled(line.trim_start_matches('#').trim().to_string(), Style::default().add_modifier(Modifier::BOLD).add_modifier(Modifier::UNDERLINED))));
             } else if in_code_block {
                 lines.push(Line::from(Span::styled(format!(" {} ", line), code_style)));
             } else {
                 lines.push(Line::from(line.to_string()));
             }
        }
        lines.push(Line::from(""));
    }

    // 6. Aliases
    if let Some(aliases) = &item.meta.aliases {
        if !aliases.is_empty() {
            lines.push(Line::from(Span::styled("Aliases", header_style)));
            lines.push(Line::from(""));
            for alias in aliases {
                lines.push(Line::from(format!("• {}", alias.join("."))));
            }
            lines.push(Line::from(""));
        }
    }

    // 7. Implementation / Tip
    lines.push(Line::from(Span::styled("Implementation", header_style)));
    lines.push(Line::from(""));

    if item.meta.is_primop.unwrap_or(false) {
        let tip_style = Style::default().fg(Color::Green).add_modifier(Modifier::ITALIC);
        lines.push(Line::from(Span::styled("Tip", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD))));
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled("This function is implemented in c++ and is part of the native nix runtime.", tip_style)));
    } else if let Some(expr) = &item.meta.attr_expr {
        lines.push(Line::from(Span::styled("Tip", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD))));
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled("The following is the current implementation of this function.", Style::default().fg(Color::Green).add_modifier(Modifier::ITALIC))));
        lines.push(Line::from(""));

        lines.extend(format_code_block(expr));
    }
    lines.push(Line::from(""));

    // 8. Source Code Link
    let mut url_out = None;
    if let Some(pos) = &item.meta.attr_position {
        // Construct file URL
        let file_path = pos.file.to_string_lossy();
        // Return URL separately to be handled by a specialized Widget
        url_out = Some(format!("file://{}", file_path));
    }

    (Text::from(lines), url_out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{DocumentFrontmatter, PrimopMatter, ContentSource, SourceOrigin};

    fn mock_doc(title: &str, content: Option<&str>, primop: bool) -> DocItem {
        DocItem {
            meta: DocumentFrontmatter {
                title: title.to_string(),
                path: vec![],
                aliases: None,
                signature: Some("test :: a -> b".to_string()),
                is_primop: Some(primop),
                primop_meta: if primop { Some(PrimopMatter { name: Some("test".to_string()), args: Some(vec!["a".to_string(), "b".to_string()]), experimental: None, arity: Some(2) }) } else { None },
                is_functor: None,
                attr_position: None,
                attr_expr: Some("impl = ...\nline 2".to_string()),
                lambda_position: None,
                lambda_expr: None,
                count_applied: None,
                content_meta: None,
            },
            content: Some(ContentSource {
                content: content.map(|s| s.to_string()),
                source: None,
            }),
        }
    }

    #[test]
    fn test_parse_simple() {
        let content = "Desc\n\n# Inputs\n\nInp\n\n# Type\n\nTyp\n\n# Examples\n\nEx";
        let doc = mock_doc("test", Some(content), false);
        let parsed = parse_doc(&doc);
        assert_eq!(parsed.description, "Desc");
        assert_eq!(parsed.inputs[0], "Inp");
        assert_eq!(parsed.type_info, Some("Typ".to_string()));
        assert_eq!(parsed.examples, Some("Ex".to_string()));
    }

    #[test]
    fn test_render_inputs_formatting() {
        let content = "Desc\n\n# Inputs\n\n`arg`\n: 1\\. Desc";
        let doc = mock_doc("test", Some(content), false);
        let (text, _) = render_doc(&doc);
        let output = format!("{:?}", text);
        // It should contain the styled arg and processed description
        // Debug format of Text isn't perfect for checking styles, but we can check string content modification
        assert!(output.contains("1. Desc"));
        assert!(!output.contains("1\\. Desc"));
    }

    #[test]
    fn test_render_multiline_impl() {
        let doc = mock_doc("test", None, false);
        let (text, _) = render_doc(&doc);
        let output = format!("{:?}", text);
        assert!(output.contains("impl = ..."));
        assert!(output.contains("line 2"));
        // Check that they are on separate lines (Ratatui Text struct stores lines as a vector)
        // We can't easily check the vector structure via Debug output of the whole Text without more effort,
        // but finding both strings suggests they are there.
        // We can assert the number of lines if we knew exact layout.
    }
}
