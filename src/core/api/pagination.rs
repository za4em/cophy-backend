use serde::Deserialize;

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
pub(super) struct ListQuery {
    limit: Option<i64>,
    before: Option<i64>,
}

impl ListQuery {
    pub(super) fn limit(&self, max: i64) -> i64 {
        match self.limit {
            Some(limit) if limit > 0 => limit.min(max),
            _ => max,
        }
    }

    pub(super) fn before(&self) -> Option<i64> {
        self.before
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn limit_caps_and_defaults() {
        assert_eq!(ListQuery::default().limit(100), 100);
        assert_eq!(query(Some(0)).limit(100), 100);
        assert_eq!(query(Some(-1)).limit(100), 100);
        assert_eq!(query(Some(20)).limit(100), 20);
        assert_eq!(query(Some(500)).limit(100), 100);
    }

    fn query(limit: Option<i64>) -> ListQuery {
        ListQuery {
            limit,
            before: None,
        }
    }
}
