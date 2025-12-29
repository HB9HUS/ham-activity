use uom::si::f64::Frequency;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Band {
    pub name: &'static str,
    pub lower: Frequency,
    pub upper: Frequency,
}

impl Band {
    /// Const constructor that receives the limits in kHz.
    pub const fn new_from_khz(lower_khz: u64, upper_khz: u64, name: &'static str) -> Self {
        // new does not (yet?) support const fn, hence create Frequency manually
        let lower = Frequency {
            dimension: uom::lib::marker::PhantomData,
            units: uom::lib::marker::PhantomData,
            value: (lower_khz * 1_000) as f64,
        };

        let upper = Frequency {
            dimension: uom::lib::marker::PhantomData,
            units: uom::lib::marker::PhantomData,
            value: (upper_khz * 1_000) as f64,
        };

        Self { name, lower, upper }
    }
}

pub const HF_BANDS: &[Band] = &[
    Band::new_from_khz(1_800, 2_000, "160m"),
    Band::new_from_khz(3_500, 4_000, "80m"),
    Band::new_from_khz(7_000, 7_200, "40m"),
    Band::new_from_khz(10_100, 10_150, "30m"),
    Band::new_from_khz(14_000, 14_350, "20m"),
    Band::new_from_khz(18_068, 18_168, "17m"),
    Band::new_from_khz(21_000, 21_450, "15m"),
    Band::new_from_khz(24_890, 24_990, "12m"),
    Band::new_from_khz(28_000, 29_700, "10m"),
    Band::new_from_khz(50_000, 52_000, "6m"),
];
