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

pub struct AltCompanyName<L>(pub L);

impl<L: ExtData> Dummy<AltCompanyName<L>> for String {
    fn dummy_with_rng<R: Rng + ?Sized>(_: &AltCompanyName<L>, rng: &mut R) -> Self {
        format!(
            "{}{}",
            *L::COMPANY_BEGINNING_PART.choose(rng).unwrap(),
            *L::COMPANY_END_PART.choose(rng).unwrap()
        )
    }
}
