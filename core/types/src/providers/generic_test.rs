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

use crate::providers::generic::Pagination;

#[test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_pagination_limit_and_next_page() {
    let p = Pagination::new_limit(20, 0);
    assert!(p.is_first);
    assert!(p.is_valid);
    assert_eq!(p.limit, 20);
    assert_eq!(p.offset, 0);

    let p2 = p.next_page();
    assert!(!p2.is_first);
    assert_eq!(p2.offset, 20);

    let mut p3 = p2.next_page();
    assert_eq!(p3.offset, 40);

    p3.invalidate();
    assert!(!p3.is_valid);
}

#[test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_pagination_token_and_next_page_wtoken() {
    let p = Pagination::new_token(Some("tok1".to_string()));
    assert_eq!(p.token, Some("tok1".to_string()));

    let p2 = p.next_page_wtoken(Some("tok2".to_string()));
    assert_eq!(p2.token, Some("tok2".to_string()));
    assert!(!p2.is_first);
}
