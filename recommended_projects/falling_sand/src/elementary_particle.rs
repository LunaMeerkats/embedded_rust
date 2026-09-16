#[derive(Debug)]
pub(crate) struct ParticleProperties {
    pub(crate) mass: f64,      // MeV/c²
    pub(crate) twice_spin: u8, // Stores 2s: 1 means spin ½
    pub(crate) charge_thirds: i8,    // Charge in units of e/3; 2 means +2e/3.
}


pub(crate) trait ElementaryParticle {
    fn properties(&self) -> &ParticleProperties;
    
    fn mass(&self) -> f64 {
        self.properties().mass
    }

    fn spin(&self) -> f64{
        f64::from(self.properties().twice_spin) / 2.0
    }

    fn charge(&self) -> f64 {
        f64::from(self.properties().charge_thirds) / 3.0
    }
}
