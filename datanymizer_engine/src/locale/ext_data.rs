use fake::locales::{Data, EN, ZH_TW};

pub trait ExtData: Data + Copy {
    const NAME_MIDDLE_NAME: &'static [&'static str] = Self::NAME_FIRST_NAME;
    const COMPANY_BEGINNING_PART: &'static [&'static str] = Self::COMPANY_BUZZWORD_HEAD;
    const COMPANY_END_PART: &'static [&'static str] = Self::COMPANY_INDUSTRY;
}

impl ExtData for EN {}
impl ExtData for ZH_TW {}
