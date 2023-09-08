pub struct UniversalisBuilder {
    pub homeworld: String,
    pub data_centers: Vec<String>,
}

impl UniversalisBuilder {
    pub fn new(homeworld: &str, data_centers: &[String]) -> Self {
        Self {
            homeworld: homeworld.into(),
            data_centers: data_centers.into(),
        }
    }

    #[allow(dead_code)]
    pub fn homeworld<S: AsRef<str>>(mut self, homeworld: S) -> Self {
        self.homeworld = homeworld.as_ref().to_string();
        self
    }

    pub fn data_centers<S: AsRef<str>>(mut self, data_centers: Vec<S>) -> Self {
        self.data_centers = data_centers
            .into_iter()
            .map(|dc| dc.as_ref().to_string())
            .collect();
        self
    }
}
