use super::selection::update_selection;

#[test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_normal_click_selects_single_item() {
    let current = vec![1, 2, 3];
    let clicked = 5;
    let anchor = 1;

    let result = update_selection(&current, clicked, anchor, false, false, false, 10);

    assert_eq!(result, vec![5]);
}

#[test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_ctrl_click_adds_unselected_item() {
    let current = vec![1, 3];
    let clicked = 2;
    let anchor = 1;

    let result = update_selection(&current, clicked, anchor, true, false, false, 10);

    assert_eq!(result, vec![1, 2, 3]);
}

#[test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_ctrl_click_removes_selected_item() {
    let current = vec![1, 2, 3];
    let clicked = 2;
    let anchor = 1;

    let result = update_selection(&current, clicked, anchor, true, false, false, 10);

    assert_eq!(result, vec![1, 3]);
}

#[test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_shift_click_selects_forward_range() {
    let current = vec![1];
    let clicked = 4;
    let anchor = 1;

    let result = update_selection(&current, clicked, anchor, false, true, false, 10);

    assert_eq!(result, vec![1, 2, 3, 4]);
}

#[test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_shift_click_selects_backward_range() {
    let current = vec![4];
    let clicked = 1;
    let anchor = 4;

    let result = update_selection(&current, clicked, anchor, false, true, false, 10);

    assert_eq!(result, vec![1, 2, 3, 4]);
}

#[test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_right_click_adds_unselected_item() {
    let current = vec![1, 2];
    let clicked = 4;
    let anchor = 1;

    let result = update_selection(&current, clicked, anchor, false, false, true, 10);

    assert_eq!(result, vec![1, 2, 4]);
}

#[test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_right_click_preserves_already_selected_item() {
    let current = vec![1, 2, 3];
    let clicked = 2;
    let anchor = 1;

    let result = update_selection(&current, clicked, anchor, false, false, true, 10);

    assert_eq!(result, vec![1, 2, 3]);
}

#[test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_out_of_bounds_click_returns_current() {
    let current = vec![1, 2];
    let clicked = 15;
    let anchor = 1;

    let result = update_selection(&current, clicked, anchor, false, false, false, 10);

    assert_eq!(result, vec![1, 2]);
}
