/// Position in some coordinates, either latitude and longitude or local projected coordinate system.
#[derive(Debug, Clone, Copy, Default)]
pub struct Position(geo_types::Point);

impl Position {
    /// Construct from latitude and longitude.
    pub fn from_lat_lon(lat: f64, lon: f64) -> Self {
        Self(geo_types::Point::new(lon, lat))
    }

    /// Construct from longitude and latitude. Note that it is common standard to write coordinates
    /// starting with the latitude instead (e.g. `51.104465719934176, 17.075169894118684` is
    /// the [Wrocław's zoo](https://zoo.wroclaw.pl/en/)).
    pub fn from_lon_lat(lon: f64, lat: f64) -> Self {
        Self(geo_types::Point::new(lon, lat))
    }

    pub fn new(x: f64, y: f64) -> Self {
        Self(geo_types::Point::new(x, y))
    }

    pub fn x(&self) -> f64 {
        self.0.x()
    }

    pub fn y(&self) -> f64 {
        self.0.y()
    }

    pub fn lat(&self) -> f64 {
        self.0.y()
    }

    pub fn lon(&self) -> f64 {
        self.0.x()
    }
}

impl From<geo_types::Point> for Position {
    fn from(value: geo_types::Point) -> Self {
        Self(value)
    }
}

impl From<Position> for geo_types::Point {
    fn from(value: Position) -> Self {
        value.0
    }
}
