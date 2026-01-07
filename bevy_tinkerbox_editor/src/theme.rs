pub mod colors {
    use bevy::color::Srgba;
    // light-ish grey #36373b
    // highlight grey #46474d
    // dusty black #1e1e1e

    //pub const GRAY1: Srgba = Srgba::new(0.224, 0.224, 0.243, 1.0);
    pub const GRAY2: Srgba = Srgba::new(0.486, 0.486, 0.529, 1.0);
    pub const WHITE: Srgba = Srgba::new(1.0, 1.0, 1.0, 1.0);
    #[allow(dead_code)]
    pub fn gry_drk() -> Srgba {
        Srgba::hex("#1e1e1e").unwrap_or(WHITE)
    }
    pub fn gry_nut() -> Srgba {
        Srgba::hex("#36373b").unwrap_or(WHITE)
    }
    #[allow(dead_code)]
    pub fn gry_lit() -> Srgba {
        Srgba::hex("#46474d").unwrap_or(WHITE)
    }
}
