// Moosync
// Copyright (C) 2024, 2025  Moosync <support@moosync.app>
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <http://www.gnu.org/licenses/>.

use assertables::assert_some_eq_x;
use tracing_test::traced_test;

use crate::providers::generic::Pagination;

#[test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_pagination_limit_initial() {
    let pagination = Pagination::new_limit(20, 0);

    assert!(pagination.is_first);
    assert!(pagination.is_valid);
    assert_eq!(pagination.limit, 20);
    assert_eq!(pagination.offset, 0);
}

#[test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_pagination_limit_next_page() {
    let pagination = Pagination::new_limit(20, 0);

    let next = pagination.next_page();
    let second_next = next.next_page();

    assert!(!next.is_first);
    assert_eq!(next.offset, 20);
    assert_eq!(second_next.offset, 40);
}

#[test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_pagination_invalidate() {
    let mut pagination = Pagination::new_limit(20, 0);

    pagination.invalidate();

    assert!(!pagination.is_valid);
}

#[test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_pagination_token_and_next_page_wtoken() {
    let pagination = Pagination::new_token(Some("tok1".to_string()));

    let next = pagination.next_page_wtoken(Some("tok2".to_string()));

    assert_some_eq_x!(pagination.token.as_deref(), "tok1");
    assert_some_eq_x!(next.token.as_deref(), "tok2");
    assert!(!next.is_first);
}
