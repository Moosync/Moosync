#![allow(clippy::too_many_arguments)]

use rstest::rstest;
use tracing_test::traced_test;

use super::selection::update_selection;

#[rstest]
#[case(vec![1, 2, 3], 5, 1, false, false, false, 10, vec![5])]
#[case(vec![1, 3], 2, 1, true, false, false, 10, vec![1, 2, 3])]
#[case(vec![1, 2, 3], 2, 1, true, false, false, 10, vec![1, 3])]
#[case(vec![1], 4, 1, false, true, false, 10, vec![1, 2, 3, 4])]
#[case(vec![4], 1, 4, false, true, false, 10, vec![1, 2, 3, 4])]
#[case(vec![1, 2], 4, 1, false, false, true, 10, vec![1, 2, 4])]
#[case(vec![1, 2, 3], 2, 1, false, false, true, 10, vec![1, 2, 3])]
#[case(vec![1, 2], 15, 1, false, false, false, 10, vec![1, 2])]
#[allow(clippy::too_many_arguments)]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_update_selection(
    #[case] current: Vec<i32>,
    #[case] clicked: i32,
    #[case] anchor: i32,
    #[case] ctrl: bool,
    #[case] shift: bool,
    #[case] right: bool,
    #[case] total_count: usize,
    #[case] expected: Vec<i32>,
) {
    let result = update_selection(&current, clicked, anchor, ctrl, shift, right, total_count);

    assert_eq!(result, expected);
}
