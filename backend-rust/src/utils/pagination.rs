pub fn normalize_page(page: Option<i64>, per_page: Option<i64>) -> (i64, i64, i64) {
    let page = page.unwrap_or(1).max(1);
    let per_page = per_page.unwrap_or(50).clamp(1, 200);
    let offset = (page - 1) * per_page;
    (page, per_page, offset)
}

pub fn normalize_limit(limit: Option<i64>, default: i64, max: i64) -> i64 {
    limit.unwrap_or(default).clamp(1, max)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_page_clamps() {
        assert_eq!(normalize_page(Some(0), Some(500)), (1, 200, 0));
        assert_eq!(normalize_page(None, None), (1, 50, 0));
        assert_eq!(normalize_page(Some(2), Some(10)), (2, 10, 10));
    }

    #[test]
    fn normalize_limit_clamps() {
        assert_eq!(normalize_limit(None, 100, 500), 100);
        assert_eq!(normalize_limit(Some(1000), 100, 500), 500);
    }
}
