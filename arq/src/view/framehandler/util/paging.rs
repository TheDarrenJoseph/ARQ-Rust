use std::convert::TryInto;
use tui::layout::Rect;
use tui::widgets::Paragraph;
use crate::list_selection::{ItemListSelection, ListSelection};
use crate::map::objects::container::Container;
use crate::view::framehandler::util::tabling::build_paragraph;

pub fn build_page_count<'a>(item_list_selection : &ItemListSelection, area: Rect) -> (Paragraph<'a>, Rect, usize) {
    let page_number = item_list_selection.get_page_number();
    let total_pages = item_list_selection.get_total_pages();
    let item_count = item_list_selection.get_items().len();
    let page_count_text = format!("Page {}/{} ({})", page_number, total_pages, item_count);
    let page_count_text_length = page_count_text.len();
    let width = page_count_text.len().try_into().unwrap();
    let page_count_paragraph = build_paragraph(page_count_text);
    let page_count_area = Rect::new( area.width.clone() - page_count_text_length as u16 , area.y.clone() + area.height.clone() - 1, width, 1);
    (page_count_paragraph, page_count_area, page_count_text_length)
}

pub fn build_weight_limit<'a>(container : &Container, area: Rect, x_offset: usize) -> (Paragraph<'a>, Rect) {
    let weight_limit = container.get_weight_limit();
    let container_item_weight_total = container.get_contents_weight_total();
    let weight_limit_text = format!("{}/{}Kg", container_item_weight_total, weight_limit);
    let weight_limit_text_length = weight_limit_text.len();
    let weight_limit_text_width = weight_limit_text.len().try_into().unwrap();
    let weight_limit_paragraph = build_paragraph(weight_limit_text);
    let weight_limit_area = Rect::new( area.width.clone() - x_offset as u16 - weight_limit_text_length as u16 - 1, area.y.clone() + area.height.clone() - 1, weight_limit_text_width, 1);
    (weight_limit_paragraph, weight_limit_area)
}