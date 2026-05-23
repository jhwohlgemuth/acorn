/// Blackwell models that map to compute capability 10.3.
pub const NVIDIA_CC_BLACKWELL_103_PATTERNS: &[&str] = &["gb300", "b300"];
/// Blackwell models that map to compute capability 10.0.
pub const NVIDIA_CC_BLACKWELL_100_PATTERNS: &[&str] = &["gb200", "b200"];
/// Ampere models that map to compute capability 8.0.
pub const NVIDIA_CC_AMPERE_80_PATTERNS: &[&str] = &["a100", "a30"];
/// Pascal models that map to compute capability 6.0.
pub const NVIDIA_CC_PASCAL_60_PATTERNS: &[&str] = &["p100", "gp100"];
/// Maxwell models that map to compute capability 5.2.
pub const NVIDIA_CC_MAXWELL_52_PATTERNS: &[&str] = &["m60", "m40"];
/// Maxwell models that map to compute capability 5.0.
pub const NVIDIA_CC_MAXWELL_50_PATTERNS: &[&str] = &[
    "k2200", "k1200", "k620", "m1200", "m520", "m5000m", "m4000m", "m3000m", "m2000m", "m1000m", "m600m", "m500m", "nvs810", "960m", "950m", "940m",
    "930m", "850m", "840m", "830m",
];
/// Kepler models that map to compute capability 3.5.
pub const NVIDIA_CC_KEPLER_35_PATTERNS: &[&str] = &["k40", "k20"];
/// Kepler models that map to compute capability 3.2.
pub const NVIDIA_CC_KEPLER_32_PATTERNS: &[&str] = &["tk1", "tegrak1"];
/// Fermi models that map to compute capability 2.1 with direct matching.
pub const NVIDIA_CC_FERMI_21_PATTERNS: &[&str] = &[
    "nvs", "5400m", "5200m", "4200m", "4000m", "3000m", "2000m", "1000m", "gtx560", "gtx550", "gtx460", "gts450", "820m", "800m", "gtx675m",
    "gtx670m", "gtx580m", "gtx570m", "gtx560m", "635m", "630m", "625m", "720m", "620m", "710m", "705m", "610m", "gt555m", "gt550m", "gt540m",
    "gt525m", "gt520mx", "gt520m", "gtx485m", "gtx470m", "gtx460m", "gt445m", "gt435m", "gt420m", "gt415m", "410m",
];
/// Fermi Quadro suffixes that map to compute capability 2.1.
pub const NVIDIA_CC_FERMI_21_QUADRO_SUFFIXES: &[&str] = &["2000", "2000d", "600"];
/// Fermi GT suffixes that map to compute capability 2.1.
pub const NVIDIA_CC_FERMI_21_GT_SUFFIXES: &[&str] = &["730", "640", "630", "620", "610", "520", "440", "430"];
/// Tesla models that map to compute capability 1.3 with direct matching.
pub const NVIDIA_CC_TESLA_13_PATTERNS: &[&str] = &[
    "c1060", "s1070", "m1060", "cx", "plex2200", "gtx295", "gtx285", "gtx280", "gtx275", "gtx260",
];
/// Tesla Quadro suffixes that map to compute capability 1.3.
pub const NVIDIA_CC_TESLA_13_QUADRO_SUFFIXES: &[&str] = &["fx5800", "fx4800", "fx3800"];
/// Tesla models that map to compute capability 1.2 with direct matching.
pub const NVIDIA_CC_TESLA_12_PATTERNS: &[&str] = &[
    "quadro400",
    "nvs300",
    "nvs5100m",
    "nvs3100m",
    "nvs2100m",
    "g210m",
    "310m",
    "305m",
    "gts360m",
    "gts350m",
];
/// Tesla Quadro suffixes that map to compute capability 1.2.
pub const NVIDIA_CC_TESLA_12_QUADRO_SUFFIXES: &[&str] = &["fx380", "fx1800m", "fx880m", "fx380m"];
/// Tesla GT suffixes that map to compute capability 1.2.
pub const NVIDIA_CC_TESLA_12_GT_SUFFIXES: &[&str] = &["240", "220", "210", "335m", "330m", "325m", "240m"];
/// Tesla models that map to compute capability 1.1 with direct matching.
pub const NVIDIA_CC_TESLA_11_PATTERNS: &[&str] = &[
    "nvs450", "nvs420", "nvs295", "nvs320m", "nvs160m", "nvs150m", "nvs140m", "nvs135m", "nvs130m", "9800gx2", "9800gtx+", "9800gtx", "9600gso",
    "9500gt", "8800gts", "8800gt", "8800gs", "8600gts", "8600gt", "8500gt", "8400gs", "9400mgpu", "9300mgpu", "8300mgpu", "8200mgpu", "8100mgpu",
    "gtx285m", "gtx280m", "gtx260m", "9800mgtx", "8800mgtx", "gts260m", "gts250m", "9800mgt", "9600mgt", "8800mgts", "9800mgts", "gt230m", "9700mgt",
    "9650mgs", "9600mgt", "9600mgs", "9500mgs", "8700mgt", "8600mgt", "8600mgs", "9500mg", "9300mg", "8400mgs", "g210m", "g110m", "9300mgs",
    "9200mgs", "9100mg", "8400mgt", "g105m",
];
/// Tesla Quadro suffixes that map to compute capability 1.1.
pub const NVIDIA_CC_TESLA_11_QUADRO_SUFFIXES: &[&str] = &[
    "fx4700", "fx3700", "fx1800", "fx1700", "fx580", "fx570", "fx470", "fx380", "fx370", "fx3800m", "fx3700m", "fx3600m", "fx2800m", "fx2700m",
    "fx1700m", "fx1600m", "fx770m", "fx570m", "fx370m", "fx360m",
];
/// Tesla GTS suffixes that map to compute capability 1.1.
pub const NVIDIA_CC_TESLA_11_GTS_SUFFIXES: &[&str] = &["250", "150"];
/// Tesla GT suffixes that map to compute capability 1.1.
pub const NVIDIA_CC_TESLA_11_GT_SUFFIXES: &[&str] = &["130", "120", "100"];
