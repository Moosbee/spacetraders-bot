use super::nav_models::{Cache, NavMode, RouteConnection};

impl Cache {
    pub fn get(
        &self,
        start_symbol: String,
        end_symbol: String,
        nav_mode: &NavMode,
        only_markets: bool,
        range: i32,
    ) -> Option<Vec<RouteConnection>> {
        self.routes
            .get(&(
                start_symbol.clone(),
                end_symbol.clone(),
                nav_mode.clone(),
                only_markets,
                range,
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
        value: Vec<RouteConnection>,
    ) {
        let key: (String, String, NavMode, bool, i32) = (
            start_symbol.clone(),
            end_symbol.clone(),
            nav_mode.clone(),
            only_markets,
            range,
        );
        self.routes.insert(key, value);
    }
}
