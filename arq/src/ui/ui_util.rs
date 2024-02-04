
use std::io::{Error, ErrorKind};
use tui::layout::{Constraint, Direction, Layout, Rect};
use tui::style::Style;
use tui::text::{Span, Spans};
use tui::widgets::Paragraph;
use crate::map::position::Area;
use crate::ui::ui_areas::UIAreas;

const MIN_VIEW_SIZE : u16 = 3;
pub const MIN_AREA: Rect = Rect { x:0, y: 0, width: 80, height: 24};

enum Alignment {
    LEFT,
    RIGHT
}

pub fn build_paragraph_multi<'a>(messages: Vec<String>) -> Paragraph<'a> {
    let mut spans = Vec::new();
    for line in messages {
        spans.push(Spans::from(Span::raw(line.clone())));
    }
    let paragraph = Paragraph::new(spans)
        .style(Style::default())
        .alignment(tui::layout::Alignment::Left);
    paragraph
}

pub fn build_paragraph<'a>(text: String) -> Paragraph<'a> {
    let spans = vec![Spans::from(Span::raw(text.clone()))];
    let paragraph = Paragraph::new(spans)
        .style(Style::default())
        .alignment(tui::layout::Alignment::Left);
    paragraph
}

/*
    target - the expected size e.g height or width
    available - the space available
    alignment - the fallback alignment for any scenario where available does not divide evenly
 */
fn center(target: u16, available: u16, alignment: Alignment)  -> Result<u16, Error> {
    // We've already checked > target height so we only need to check these 2 scenarios
    if target == available {
        Ok(target)
    } else if target > available {
        Err(Error::new(ErrorKind::Other,
                   format!("Target size: {} is above the available range: {}", target, available)))
    } else {
        let diff = available - target;
        // Divides by 2 so we can use the same margin each side
        if diff % 2 == 0 {
            Ok(diff / 2)
        } else {
            // Otherwise top / left align
            let half_diff = diff / 2;
            let remainder = diff % 2;

            return  match alignment {
                Alignment::LEFT => {
                    Ok(half_diff)
                },
                Alignment::RIGHT => {
                    Ok(half_diff + remainder)
                }
            }
        }
    }
}

pub fn center_area(target: Rect, frame_size: Rect, min_area: Rect) -> Result<Rect, Error> {
    if target.height < min_area.height || target.width < min_area.width {
        return Err(Error::new(ErrorKind::Other,
                       format!("Target size: {}, {} is below the minimum supported range: {}, {}", target.width, target.height, min_area.width, min_area.height)))
    }

    if target.height > frame_size.height && target.width > frame_size.width {
        Err(Error::new(ErrorKind::Other,
                       format!("Target size: {}, {} is above the available range: {}, {}", target.width, target.height, min_area.width, min_area.height)))
    } else if target.height < frame_size.height || target.width < frame_size.width {
        let target_width = target.width;
        let mut x = target.x;
        if target.width < frame_size.width {
            let frame_width = frame_size.width;
            x = center(target_width, frame_width, Alignment::LEFT)?;
            x += frame_size.x;
        }

        let target_height = target.height;
        let mut y = target.y;
        if target.height < frame_size.height {
            let frame_height = frame_size.height;
            y = center(target_height, frame_height, Alignment::LEFT)?;
            y += frame_size.y;
        }
        return Ok(Rect::new(x, y, target_width, target_height))
    } else {
        // height and width match the target exactly
        return Ok(target);
    }
}

pub(crate) fn check_display_size(frame_size_result: Option<Area>) -> Result<(), std::io::Error> {
    if let Some(frame_size) = frame_size_result {
        // Check for 80x24 minimum resolution
        if frame_size.height < MIN_AREA.height || frame_size.width < MIN_AREA.width {
            return Err(Error::new(ErrorKind::Other, format!("Sorry, your terminal is below the minimum supported resolution of {}x{}.", MIN_AREA.height, MIN_AREA.width)));
        }
        Ok(())
    } else {
        return Err(Error::new(ErrorKind::Other, "Something went wrong; no frame size has been set!"));
    }
}

#[cfg(test)]
mod tests {
    
    use tui::layout::Rect;
    use crate::ui::ui_util::{center_area, MIN_AREA};

    #[test]
    fn test_center_area_div2_leftalign() {
        // GIVEN a square screen area larger than the target of 80 columns and 24
        let available = Rect::new(0,0, 100, 100);
        let target = Rect::new(0,0, 80,24);

        // WHEN we call to center for the target
        let result = center_area(target, available, MIN_AREA);
        // THEN we expect a perfectly centered result
        assert!(result.is_ok());
        let result_area = result.unwrap();

        assert_eq!(10, result_area.x);
        assert_eq!(38, result_area.y);
        assert_eq!(24, result_area.height);
        assert_eq!(80, result_area.width);
    }

    #[test]
    fn test_center_area_1over_leftalign() {
        // GIVEN a screen area larger than the target of 80 columns and 24
        // AND the screen area has 1 character extra space either side
        let available = Rect::new(0,0, 101, 101);
        let target = Rect::new(0,0, 80,24);

        // WHEN we call to center for the target
        let result = center_area(target, available, MIN_AREA);
        // THEN we expect a left-aligned result
        assert!(result.is_ok());
        let result_area = result.unwrap();

        assert_eq!(10, result_area.x);
        assert_eq!(38, result_area.y);
        assert_eq!(24, result_area.height);
        assert_eq!(80, result_area.width);
    }
}