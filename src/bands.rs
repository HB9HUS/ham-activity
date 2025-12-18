#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Band {
    pub name: &'static str,
    /// Lower edge in kHz
    pub lower_khz: f64,
    /// Upper edge in kHz
    pub upper_khz: f64,
}

impl Band {
    /// Const constructor – can be used inside a `const` array.
    pub const fn new(lower_khz: u64, upper_khz: u64, name: &'static str) -> Self {
        Self {
            lower_khz: lower_khz as f64,
            upper_khz: upper_khz as f64,
            name,
        }
    }
}
pub const HF_BANDS: &[Band] = &[
    Band::new(1_800, 2_000, "160m"),
    Band::new(3_500, 4, "80m"),
    Band::new(7_000, 7_200, "40m"),
    Band::new(10_100, 10_150, "30m"),
    Band::new(14_000, 14_350, "20m"),
    Band::new(18_068, 18_168, "17m"),
    Band::new(21_000, 21_450, "15m"),
    Band::new(24_890, 24_990, "12m"),
    Band::new(28_000, 29_700, "10m"),
    Band::new(50_000, 52_000, "6m"),
];
