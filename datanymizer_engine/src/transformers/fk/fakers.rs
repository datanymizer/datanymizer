use crate::ExtData;
use fake::Dummy;
use rand::{seq::SliceRandom, Rng};

pub struct MiddleName<L>(pub L);

impl<L: ExtData> Dummy<MiddleName<L>> for String {
    fn dummy_with_rng<R: Rng + ?Sized>(_: &MiddleName<L>, rng: &mut R) -> Self {
        let s = *L::NAME_MIDDLE_NAME.choose(rng).unwrap();
        s.into()
    }
}

pub struct CompanyNameAlt<L>(pub L);

impl<L: ExtData> Dummy<CompanyNameAlt<L>> for String {
    fn dummy_with_rng<R: Rng + ?Sized>(_: &CompanyNameAlt<L>, rng: &mut R) -> Self {
        format!(
            "{}{}",
            *L::COMPANY_BEGINNING_PART.choose(rng).unwrap(),
            *L::COMPANY_END_PART.choose(rng).unwrap()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::locale::RU;
    use fake::Fake;

    #[test]
    fn middle_name() {
        let faker = MiddleName(RU);
        let value: String = faker.fake();

        assert!(RU::NAME_MIDDLE_NAME
            .iter()
            .any(|&dict_value| dict_value == value.as_str()));
    }

    #[test]
    fn company_name_alt() {
        let faker = CompanyNameAlt(RU);
        let value: String = faker.fake();

        assert!(RU::COMPANY_BEGINNING_PART
            .iter()
            .any(|&dict_value| value.starts_with(dict_value)));
        assert!(RU::COMPANY_END_PART
            .iter()
            .any(|&dict_value| value.ends_with(dict_value)));
    }
}
