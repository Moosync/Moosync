#[tracing::instrument(level = "debug", skip_all)]
pub fn update_selection(
    current_indices: &[i32],
    clicked_idx: i32,
    anchor_idx: i32,
    is_ctrl: bool,
    is_shift: bool,
    is_right_click: bool,
    total_count: usize,
) -> Vec<i32> {
    if clicked_idx < 0 || clicked_idx as usize >= total_count {
        return current_indices.to_vec();
    }

    if is_right_click {
        if current_indices.contains(&clicked_idx) {
            return current_indices.to_vec();
        }
        let mut updated = current_indices.to_vec();
        updated.push(clicked_idx);
        updated.sort_unstable();
        return updated;
    }

    if is_shift {
        let anchor = if anchor_idx >= 0 && (anchor_idx as usize) < total_count {
            anchor_idx
        } else {
            0
        };
        let start = anchor.min(clicked_idx);
        let end = anchor.max(clicked_idx);
        let mut updated = Vec::with_capacity((end - start + 1) as usize);
        for i in start..=end {
            updated.push(i);
        }
        return updated;
    }

    if is_ctrl {
        let mut updated = current_indices.to_vec();
        if let Some(pos) = updated.iter().position(|&x| x == clicked_idx) {
            updated.remove(pos);
            return updated;
        }
        updated.push(clicked_idx);
        updated.sort_unstable();
        return updated;
    }

    vec![clicked_idx]
}
