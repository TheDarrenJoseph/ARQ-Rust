use tui::layout::{Alignment};
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


#[test]
fn test_build_headings() {
    use tui::buffer::{Buffer, Cell};
    use tui::layout::{Rect};
    use tui::widgets::{Widget};

    // GIVEN a view with a series of columns configured
    let columns = vec![
        Column {name : "NAME".to_string(), size: 12},
        Column {name : "WEIGHT (Kg)".to_string(), size: 12},
        Column {name : "VALUE".to_string(), size: 12}
    ];

    // WHEN we call to build the headings Paragraph
    let headings = build_headings(columns);

    // THEN we expect it to render to the buffer as expected
    let area = Rect { x: 0, y: 0, height: 2, width: 31 };
    let _cell_buffer : Vec<Cell> = Vec::new();
    let mut buffer = Buffer::empty(area.clone());
    headings.render(area, &mut buffer);
    assert_eq!(Buffer::with_lines(vec!["NAME         WEIGHT (Kg)  VALUE", "    "]), buffer);
}