/// A city in the browseable city list (left / right arrows).
pub struct CityEntry {
    pub name: &'static str,
    pub lat: f64,
    pub lon: f64,
    pub scene_id: &'static str,
}

/// Built-in city gallery — showcases all five scene types.
/// The user's auto-detected location is prepended at runtime as index 0.
pub const DEFAULT_CITIES: &[CityEntry] = &[
    CityEntry { name: "Santa Cruz, CA",    lat:  36.9741, lon: -122.0308, scene_id: "santa_cruz" },
    CityEntry { name: "New York, NY",      lat:  40.7128, lon:  -74.0060, scene_id: "city" },
    CityEntry { name: "London, UK",        lat:  51.5074, lon:   -0.1278, scene_id: "city" },
    CityEntry { name: "Paris, France",     lat:  48.8566, lon:    2.3522, scene_id: "city" },
    CityEntry { name: "Tokyo, Japan",      lat:  35.6762, lon:  139.6503, scene_id: "city" },
    CityEntry { name: "Miami Beach, FL",   lat:  25.7907, lon:  -80.1300, scene_id: "beach" },
    CityEntry { name: "Honolulu, HI",      lat:  21.3069, lon: -157.8583, scene_id: "beach" },
    CityEntry { name: "Maldives",          lat:   3.2028, lon:   73.2207, scene_id: "beach" },
    CityEntry { name: "Rio de Janeiro",    lat: -22.9068, lon:  -43.1729, scene_id: "beach" },
    CityEntry { name: "Denver, CO",        lat:  39.7392, lon: -104.9903, scene_id: "mountain" },
    CityEntry { name: "Zermatt, CH",       lat:  46.0207, lon:    7.7491, scene_id: "mountain" },
    CityEntry { name: "Innsbruck, AT",     lat:  47.2692, lon:   11.4041, scene_id: "mountain" },
    CityEntry { name: "Banff, AB",         lat:  51.1784, lon: -115.5708, scene_id: "mountain" },
    CityEntry { name: "Kansas, USA",       lat:  38.5266, lon:  -96.7265, scene_id: "farm" },
    CityEntry { name: "Tuscany, Italy",    lat:  43.7711, lon:   11.2486, scene_id: "farm" },
    CityEntry { name: "Sydney, AU",        lat: -33.8688, lon:  151.2093, scene_id: "city" },
    CityEntry { name: "Dubai, UAE",        lat:  25.2048, lon:   55.2708, scene_id: "city" },
    CityEntry { name: "Chicago, IL",       lat:  41.8781, lon:  -87.6298, scene_id: "city" },
    CityEntry { name: "Berlin, Germany",   lat:  52.5200, lon:   13.4050, scene_id: "city" },
    CityEntry { name: "Amsterdam, NL",     lat:  52.3676, lon:    4.9041, scene_id: "city" },
];

pub struct GalleryEntry {
    pub label: &'static str,
    pub scene_id: &'static str,
}

pub const GALLERY: &[GalleryEntry] = &[
    GalleryEntry { label: "Countryside",         scene_id: "world" },
    GalleryEntry { label: "City Skyline",         scene_id: "city" },
    GalleryEntry { label: "Farm",                 scene_id: "farm" },
    GalleryEntry { label: "Beach",                scene_id: "beach" },
    GalleryEntry { label: "Santa Cruz Boardwalk", scene_id: "santa_cruz" },
    GalleryEntry { label: "Mountain",             scene_id: "mountain" },
];

pub fn scene_for_city(city_name: &str) -> &'static str {
    let city = city_name.to_lowercase();

    if city.contains("santa cruz") {
        return "santa_cruz";
    }

    const BEACH_CITIES: &[&str] = &[
        "miami", "honolulu", "cancun", "cancún", "malibu", "nice", "cannes",
        "mykonos", "santorini", "bali", "phuket", "acapulco", "cabo",
        "rio de janeiro", "fort lauderdale", "clearwater", "santa barbara",
        "monterey", "carmel", "tel aviv", "gold coast", "surfers paradise",
        "waikiki", "kona", "maui", "kauai",
    ];
    if BEACH_CITIES.iter().any(|&b| city.contains(b)) {
        return "beach";
    }

    const MOUNTAIN_CITIES: &[&str] = &[
        "aspen", "vail", "breckenridge", "telluride", "steamboat", "boulder",
        "salt lake city", "park city", "jackson hole", "lake tahoe",
        "zurich", "bern", "interlaken", "innsbruck", "salzburg", "la paz",
        "quito", "kathmandu", "calgary", "banff", "whistler", "verbier",
        "zermatt", "chamonix", "davos", "gstaad", "courmayeur", "crans-montana",
    ];
    if MOUNTAIN_CITIES.iter().any(|&m| city.contains(m)) {
        return "mountain";
    }

    const FARM_CITIES: &[&str] = &[
        "wichita", "topeka", "ames", "des moines", "sioux falls", "bismarck",
        "fargo", "lincoln", "omaha", "peoria", "springfield", "champaign",
        "tuscany", "toscana", "emilia", "umbria",
    ];
    if FARM_CITIES.iter().any(|&f| city.contains(f)) {
        return "farm";
    }

    const URBAN_CITIES: &[&str] = &[
        "new york", "london", "paris", "tokyo", "chicago", "shanghai", "beijing",
        "los angeles", "dubai", "singapore", "seoul", "sydney", "toronto", "berlin",
        "madrid", "rome", "amsterdam", "vienna", "prague", "budapest", "warsaw",
        "stockholm", "oslo", "helsinki", "copenhagen", "brussels", "lisbon",
        "athens", "istanbul", "moscow", "cairo", "mumbai", "delhi", "bangalore",
        "bangkok", "hong kong", "taipei", "osaka", "montreal", "vancouver",
        "seattle", "portland", "san francisco", "houston", "dallas", "atlanta",
        "boston", "philadelphia", "phoenix", "detroit", "minneapolis",
        "rio", "sao paulo", "buenos aires", "lima", "bogota", "santiago",
        "mexico city", "nairobi", "johannesburg", "lagos", "accra", "casablanca",
        "kuala lumpur", "jakarta", "manila", "denver",
    ];
    if URBAN_CITIES.iter().any(|&u| city.contains(u)) {
        return "city";
    }

    "world"
}
