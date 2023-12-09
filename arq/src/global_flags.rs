pub struct GlobalTestFlags {
    pub debugging_map_symbols: bool,
    pub rng_seed: Option<&'static str>
}

pub const GLOBALS: GlobalTestFlags = GlobalTestFlags {
    debugging_map_symbols: false,
    rng_seed: None //Some("02sZFl3vcYKb")
};