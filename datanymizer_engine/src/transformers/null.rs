use crate::transformer::{TransformContext, TransformResult, TransformResultHelper, Transformer};
use serde::{Deserialize, Serialize};

/// Sets the field value to NULL (`\N` in PostgreSQL COPY format).
#[derive(Serialize, Deserialize, PartialEq, Eq, Hash, Clone, Debug)]
pub struct NullTransformer;

impl Transformer for NullTransformer {
    fn transform(
        &self,
        _field_name: &str,
        _field_value: &str,
        _ctx: &Option<TransformContext>,
    ) -> TransformResult {
        TransformResult::present("\\N")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sets_value_to_null() {
        let transformer: NullTransformer = serde_yaml::from_str("~").unwrap();
        let value = transformer
            .transform("field", "some_value", &None)
            .unwrap()
            .unwrap();

        assert_eq!(value, "\\N");
    }
}
