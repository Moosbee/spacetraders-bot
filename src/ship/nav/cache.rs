use super::nav_models::{Cache, NavMode, RouteConnection};

impl Cache {
    pub fn get(
        &self,
        start_symbol: String,
        end_symbol: String,
        nav_mode: &NavMode,
        only_markets: bool,
        range: i32,
        start_range: i32,
    ) -> Option<Vec<RouteConnection>> {
        self.routes
            .get(&(
                start_symbol.clone(),
                end_symbol.clone(),
                *nav_mode,
                only_markets,
                range,
                start_range,
            ))
            .cloned()
        // None
    }

    pub fn put(
        &mut self,
        start_symbol: String,
        end_symbol: String,
        nav_mode: &NavMode,
        only_markets: bool,
        range: i32,
        start_range: i32,
        value: Vec<RouteConnection>,
    ) {
        let key: (String, String, NavMode, bool, i32, i32) = (
            start_symbol.clone(),
            end_symbol.clone(),
            *nav_mode,
            only_markets,
            start_range,
            range,
        );
        self.routes.insert(key, value);
    }
}
