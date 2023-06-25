use ratatui::widgets::ListState;

pub fn get_next_index(list_state: &ListState) -> Option<usize> {
    let next_index = list_state.selected().map_or(0, |i| i + 1);
    Some(next_index)
}

pub fn get_previous_index(list_state: &ListState) -> Option<usize> {
    let next_index = list_state.selected().map_or(0, |i| i.saturating_sub(1));
    Some(next_index)
}
