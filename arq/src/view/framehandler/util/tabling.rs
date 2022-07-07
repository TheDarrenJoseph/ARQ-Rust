use tui::layout::Alignment;
use tui::style::Style;
use tui::text::{Span, Spans};
use tui::widgets::{Block, Borders, Paragraph, Wrap};

#[derive(Clone)]
pub struct Column {
    pub name : String,
    pub size : i8
}

pub fn build_padding(length : i8) -> String {
    let mut s = String::new();
    for _i in 1..length {
        s.push(' ');
    }
    s
}

pub fn build_headings<'a>(columns : Vec<Column>) -> Paragraph<'a> {
    let mut heading_spans = Vec::new();
    let mut spans = Vec::new();
    for column in columns {
        let name = column.name.clone();
        let padding = build_padding(column.size - name.len() as i8 + 2);
        spans.push(Span::raw(column.name.clone()));
        spans.push(Span::raw(padding));
    }
    heading_spans.push(Spans::from(spans));
    Paragraph::new(heading_spans)
        .block(Block::default()
            .borders(Borders::NONE))
        .style(Style::default())
        .alignment(Alignment::Left)
        .wrap(Wrap { trim: false })
}

pub fn build_paragraph<'a>(text: String) -> Paragraph<'a> {
    let spans = vec![Spans::from(Span::raw(text.clone()))];
    let paragraph = Paragraph::new(spans)
        .style(Style::default())
        .alignment(Alignment::Left);
    paragraph
}
