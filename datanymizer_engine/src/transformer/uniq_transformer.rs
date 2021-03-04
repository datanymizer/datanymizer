use super::{TransformContext, TransformResult, TransformResultHelper, Transformer, Uniqueness};
use crate::uniq_collector;

pub trait UniqTransformer {
    fn do_transform(
        &self,
        field_name: &str,
        field_value: &str,
        ctx: &Option<TransformContext>,
    ) -> String;

    fn transform_with_retry(
        &self,
        field_name: &str,
        field_value: &str,
        ctx: &Option<TransformContext>,
    ) -> Option<String> {
        let mut count = self.try_count();
        while count > 0 {
            let val = self.do_transform(field_name, field_value, ctx);
            if uniq_collector::add_to_collector(&field_name, &val) {
                return Some(val);
            } else {
                count -= 1;
            }
        }
        None
    }

    fn uniq(&self) -> &Uniqueness;

    fn try_count(&self) -> i64 {
        self.uniq()
            .try_count
            .unwrap_or_else(|| self.default_try_count())
    }

    fn default_try_count(&self) -> i64 {
        3
    }

    fn try_limit_message(&self, field_name: &str) -> String {
        format!(
            "field: `{}` with retry limit: `{}` exceeded",
            field_name,
            self.try_count()
        )
    }
}

impl<T> Transformer for T
where
    T: UniqTransformer,
{
    fn transform(
        &self,
        field_name: &str,
        field_value: &str,
        ctx: &Option<TransformContext>,
    ) -> TransformResult {
        if self.uniq().required {
            match self.transform_with_retry(field_name, field_value, ctx) {
                Some(val) => TransformResult::present(val),
                None => TransformResult::error(
                    field_name,
                    field_value,
                    &self.try_limit_message(field_name),
                ),
            }
        } else {
            TransformResult::present(self.do_transform(field_name, field_value, ctx))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockTransformer {
        default_try_count: i64,
        uniq: Uniqueness,
    }

    impl UniqTransformer for MockTransformer {
        fn do_transform(
            &self,
            _field_name: &str,
            _field_value: &str,
            _ctx: &Option<TransformContext>,
        ) -> String {
            Self::transformed_value()
        }

        fn uniq(&self) -> &Uniqueness {
            &self.uniq
        }

        fn default_try_count(&self) -> i64 {
            self.default_try_count
        }
    }

    impl MockTransformer {
        fn transformed_value() -> String {
            String::from("abc")
        }
    }

    fn assert_ok_result(result: TransformResult) {
        assert_eq!(result, Ok(Some(MockTransformer::transformed_value())));
    }

    fn assert_err_limit(result: TransformResult, limit: i64) {
        assert!(result.is_err());

        if let Err(error) = result {
            assert_eq!(
                error.reason,
                format!(
                    "field: `{}` with retry limit: `{}` exceeded",
                    error.field_name, limit
                )
            );
        };
    }

    #[test]
    fn no_uniqueness() {
        let transformer = MockTransformer {
            default_try_count: 0,
            uniq: Uniqueness {
                required: false,
                try_count: None,
            },
        };
        let name = "uniq_transformer.no_uniqueness.name";

        assert_ok_result(transformer.transform(name, "val", &None));
        assert_ok_result(transformer.transform(name, "val", &None));
    }

    #[test]
    fn uniqueness_no_retries() {
        let transformer = MockTransformer {
            default_try_count: 1,
            uniq: Uniqueness {
                required: true,
                try_count: None,
            },
        };
        let name = "uniq_transformer.uniqueness_no_retries.name";

        assert_ok_result(transformer.transform(name, "val", &None));
        assert_err_limit(transformer.transform(name, "val", &None), 1);
    }

    #[test]
    fn uniqueness_diff_fields() {
        let transformer = MockTransformer {
            default_try_count: 1,
            uniq: Uniqueness {
                required: true,
                try_count: None,
            },
        };
        let name1 = "uniq_transformer.uniqueness_diff_fields.name1";
        let name2 = "uniq_transformer.uniqueness_diff_fields.name2";

        assert_ok_result(transformer.transform(name1, "val", &None));
        assert_ok_result(transformer.transform(name2, "val", &None));
    }

    #[test]
    fn uniqueness_one_retry() {
        let transformer = MockTransformer {
            default_try_count: 2,
            uniq: Uniqueness {
                required: true,
                try_count: None,
            },
        };
        let name = "uniq_transformer.uniqueness_one_retry.name";

        assert_ok_result(transformer.transform(name, "val", &None));
        assert_err_limit(transformer.transform(name, "val", &None), 2);
    }

    #[test]
    fn uniqueness_zero_limit() {
        let transformer = MockTransformer {
            default_try_count: 0,
            uniq: Uniqueness {
                required: true,
                try_count: None,
            },
        };
        let name = "uniq_transformer.uniqueness_zero_limit.name";

        assert_err_limit(transformer.transform(name, "val", &None), 0);
    }

    #[test]
    fn limit_from_uniq() {
        let transformer = MockTransformer {
            default_try_count: 0,
            uniq: Uniqueness {
                required: true,
                try_count: Some(1),
            },
        };
        let name = "uniq_transformer.limit_from_uniq.name";

        assert_ok_result(transformer.transform(name, "val", &None));
        assert_err_limit(transformer.transform(name, "val", &None), 1);
    }
}
