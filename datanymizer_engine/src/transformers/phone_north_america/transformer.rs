use crate::transformer::{TransformContext, TransformError, Transformer};
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::sync::Mutex;

#[derive(Serialize, Deserialize, PartialEq, Eq, Hash, Debug, Clone, Default)]
pub struct PhoneNorthAmericaAreaCodeTransformer {}

impl Transformer for PhoneNorthAmericaAreaCodeTransformer {
    fn transform(
        &self,
        _field_name: &str,
        _field_value: &str,
        _ctx: &Option<TransformContext>,
    ) -> Result<Option<String>, TransformError> {
        let mut rng = rand::thread_rng();
        let area_code = REAL_AREA_CODES[rng.gen_range(0..REAL_AREA_CODES.len())];
        Ok(Some(format!("{}", area_code)))
    }
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Hash, Debug, Clone, Default)]
pub struct PhoneNorthAmericaTransformer {
    pub middle555: Option<bool>,
    pub real_area_code: Option<bool>,
}

impl Transformer for PhoneNorthAmericaTransformer {
    fn transform(
        &self,
        _field_name: &str,
        _field_value: &str,
        _ctx: &Option<TransformContext>,
    ) -> Result<Option<String>, TransformError> {
        let (lower, mut inc_upper) = {
            let mut state_lower_digits = STATE_LOWER_DIGITS.lock().unwrap();
            let existing_lower = *state_lower_digits;
            let inc_upper = existing_lower == 9999;
            if inc_upper {
                *state_lower_digits = 0;
            } else {
                *state_lower_digits += 1;
            }
            (existing_lower, inc_upper)
        };

        let middle = if self.middle555.unwrap_or_default() {
            555
        } else {
            let mut state_middle_digits = STATE_MIDDLE_DIGITS.lock().unwrap();
            let existing_middle = *state_middle_digits;
            // middle
            if inc_upper {
                inc_upper = existing_middle == 999;
                if inc_upper {
                    *state_middle_digits = 0
                } else {
                    *state_middle_digits += 1;
                }
            }
            existing_middle
        };

        let area_code = {
            let mut state_area_code = STATE_AREA_CODE.lock().unwrap();
            let existing_area_code = if *state_area_code != 0 {
                *state_area_code
            } else if self.real_area_code.unwrap_or_default() {
                REAL_AREA_CODES[0]
            } else {
                100
            };

            if inc_upper {
                if !self.real_area_code.unwrap_or_default() {
                    *state_area_code = (existing_area_code + 1) % 1000;
                } else {
                    *state_area_code = match REAL_AREA_CODES
                        .iter()
                        .position(|code| *code == existing_area_code)
                    {
                        None => REAL_AREA_CODES[0],
                        Some(existing_index) => {
                            if existing_index == REAL_AREA_CODES.len() - 1 {
                                REAL_AREA_CODES[0]
                            } else {
                                REAL_AREA_CODES[existing_index + 1]
                            }
                        }
                    }
                }
            }
            existing_area_code
        };

        Ok(Some(format!("+1-{area_code}-{middle:03}-{lower:04}")))
    }
}

static STATE_LOWER_DIGITS: Mutex<u32> = Mutex::new(0);
static STATE_MIDDLE_DIGITS: Mutex<u32> = Mutex::new(0);
static STATE_AREA_CODE: Mutex<u32> = Mutex::new(0);

#[cfg(test)]
mod tests {
    use crate::{
        transformers::PhoneNorthAmericaAreaCodeTransformer,
        transformers::PhoneNorthAmericaTransformer, utils::EnumWrapper, Transformer, Transformers,
    };
    use serial_test::serial;

    #[test]
    fn parse_config_to_phone_north_america_area_code_transformer() {
        let config = r#"
        phone_north_america_area_code: {}
        "#;
        let transformer: Transformers = EnumWrapper::parse(config).unwrap();
        assert!(matches!(
            transformer,
            Transformers::PhoneNorthAmericaAreaCode(PhoneNorthAmericaAreaCodeTransformer {})
        ));
    }

    #[test]
    #[serial]
    fn generate_phone_north_america_area_code() {
        reset_state();
        let config = r#"
        phone_north_america_area_code: {}
        "#;

        let transformer: Transformers = EnumWrapper::parse(config).unwrap();

        let val1 = transformer.transform("field", "value", &None);
        let val2 = transformer.transform("field", "value", &None);

        assert_ne!(val1, val2);
    }

    fn reset_state() {
        let mut state_lower_digits = super::STATE_LOWER_DIGITS.lock().unwrap();
        let mut state_middle_digits = super::STATE_MIDDLE_DIGITS.lock().unwrap();
        let mut state_area_code = super::STATE_AREA_CODE.lock().unwrap();
        println!("lower was {}", *state_lower_digits);
        *state_lower_digits = 0;
        println!("lower set to 0 {}", *state_lower_digits);
        *state_middle_digits = 0;
        *state_area_code = 0;
    }

    #[test]
    fn parse_config_to_phone_north_america_transformer() {
        let config = r#"
        phone_north_america:
            real_area_code: true
            middle555: true
        "#;

        let transformer: Transformers = EnumWrapper::parse(config).unwrap();
        assert!(matches!(
            transformer,
            Transformers::PhoneNorthAmerica(PhoneNorthAmericaTransformer {
                middle555: Some(true),
                real_area_code: Some(true),
            })
        ));
    }

    #[test]
    #[serial]
    fn generate_phone_north_america_real_area_code() {
        reset_state();
        let config = r#"
        phone_north_america:
            real_area_code: true
        "#;

        let transformer: Transformers = EnumWrapper::parse(config).unwrap();

        let val1 = transformer.transform("field", "value", &None);
        let val2 = transformer.transform("field", "value", &None);

        assert_eq!(val1.unwrap().unwrap(), "+1-201-000-0000");
        assert_eq!(val2.unwrap().unwrap(), "+1-201-000-0001");

        for i in 0..10000 {
            let val = transformer.transform("field", "value", &None);
            let phone = val.unwrap().unwrap();
            if i == 1000 {
                assert_eq!(phone, "+1-201-000-1002");
            }
            if i == 9999 {
                assert_eq!(phone, "+1-201-001-0001");
            }
        }
    }

    #[test]
    #[serial]
    fn generate_phone_north_america_middle555() {
        reset_state();
        let config = r#"
        phone_north_america:
            real_area_code: true
            middle555: true
        "#;

        let transformer: Transformers = EnumWrapper::parse(config).unwrap();

        let val1 = transformer.transform("field", "value", &None);
        let val2 = transformer.transform("field", "value", &None);

        assert_eq!(val1.unwrap().unwrap(), "+1-201-555-0000");
        assert_eq!(val2.unwrap().unwrap(), "+1-201-555-0001");

        for i in 0..10000 {
            let val = transformer.transform("field", "value", &None);
            let phone = val.unwrap().unwrap();
            if i == 1000 {
                assert_eq!(phone, "+1-201-555-1002");
            }
            if i == 9999 {
                assert_eq!(phone, "+1-202-555-0001");
            }
        }
    }
}

static REAL_AREA_CODES: [u32; 406] = [
    201, 202, 203, 204, 205, 206, 207, 208, 209, 210, 212, 213, 214, 215, 216, 217, 218, 219, 220,
    223, 224, 225, 226, 228, 229, 231, 234, 236, 239, 240, 242, 246, 248, 249, 250, 251, 252, 253,
    254, 256, 260, 262, 264, 267, 268, 269, 270, 272, 276, 279, 281, 284, 289, 301, 302, 303, 304,
    305, 306, 307, 308, 309, 310, 312, 313, 314, 315, 316, 317, 318, 319, 320, 321, 323, 325, 330,
    331, 332, 334, 336, 337, 339, 340, 343, 345, 346, 347, 351, 352, 360, 361, 364, 365, 367, 380,
    385, 386, 401, 402, 403, 404, 405, 406, 407, 408, 409, 410, 412, 413, 414, 415, 416, 417, 418,
    419, 423, 424, 425, 430, 431, 432, 434, 435, 437, 438, 440, 441, 442, 443, 445, 450, 456, 458,
    463, 469, 470, 473, 475, 478, 479, 480, 484, 500, 501, 502, 503, 504, 505, 506, 507, 508, 509,
    510, 512, 513, 514, 515, 516, 517, 518, 519, 520, 521, 522, 530, 531, 533, 534, 539, 540, 541,
    544, 548, 551, 559, 561, 562, 563, 564, 566, 567, 570, 571, 573, 574, 575, 577, 579, 580, 581,
    585, 586, 587, 588, 600, 601, 602, 603, 604, 605, 606, 607, 608, 609, 610, 612, 613, 614, 615,
    616, 617, 618, 619, 620, 622, 623, 626, 628, 629, 630, 631, 636, 639, 640, 641, 646, 647, 649,
    650, 651, 657, 660, 661, 662, 664, 667, 669, 670, 671, 678, 680, 681, 682, 684, 700, 701, 702,
    703, 704, 705, 706, 707, 708, 709, 710, 712, 713, 714, 715, 716, 717, 718, 719, 720, 721, 724,
    725, 726, 727, 731, 732, 734, 737, 740, 743, 747, 754, 757, 758, 760, 762, 763, 765, 767, 769,
    770, 772, 773, 774, 775, 778, 779, 780, 781, 782, 784, 785, 786, 787, 800, 801, 802, 803, 804,
    805, 806, 807, 808, 809, 810, 812, 813, 814, 815, 816, 817, 818, 819, 820, 825, 828, 829, 830,
    831, 832, 833, 838, 843, 844, 845, 847, 848, 849, 850, 854, 855, 856, 857, 858, 859, 860, 862,
    863, 864, 865, 866, 867, 868, 869, 870, 872, 873, 876, 877, 878, 888, 900, 901, 902, 903, 904,
    905, 906, 907, 908, 909, 910, 912, 913, 914, 915, 916, 917, 918, 919, 920, 925, 928, 929, 930,
    931, 934, 936, 937, 938, 939, 940, 941, 947, 949, 951, 952, 954, 956, 959, 970, 971, 972, 973,
    978, 979, 980, 984, 985, 986, 989,
];
