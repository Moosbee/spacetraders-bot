use std::num::NonZeroU32;
use std::sync::Arc;

use crate::apis::accounts_api::{GetMyAccountError, RegisterError};
use crate::apis::agents_api::{
    GetAgentError, GetAgentsError, GetMyAgentError, GetMyAgentEventsError,
};
use crate::apis::configuration::Configuration;
use crate::apis::contracts_api::{
    AcceptContractError, DeliverContractError, FulfillContractError, GetContractError,
    GetContractsError,
};
use crate::apis::factions_api::{GetFactionError, GetFactionsError};
use crate::apis::fleet_api::{
    self, CreateChartError, CreateShipShipScanError, CreateShipSystemScanError,
    CreateShipWaypointScanError, CreateSurveyError, DockShipError, ExtractResourcesError,
    ExtractResourcesWithSurveyError, GetMountsError, GetMyShipCargoError, GetMyShipError,
    GetMyShipsError, GetRepairShipError, GetScrapShipError, GetShipCooldownError,
    GetShipModulesError, GetShipNavError, InstallMountError, InstallShipModuleError, JettisonError,
    JumpShipError, NavigateShipError, OrbitShipError, PatchShipNavError, PurchaseCargoError,
    PurchaseShipError, RefuelShipError, RemoveMountError, RemoveShipModuleError, RepairShipError,
    ScrapShipError, SellCargoError, ShipRefineError, SiphonResourcesError, TransferCargoError,
    WarpShipError,
};
use crate::apis::global_api::GetStatusError;
use crate::apis::systems_api::{
    GetConstructionError, GetJumpGateError, GetMarketError, GetShipyardError, GetSystemError,
    GetSystemWaypointsError, GetSystemsError, GetWaypointError, SupplyConstructionError,
};
use crate::apis::{Error, ResponseContent, ResponseContentEntity};
use crate::models::{self, FactionSymbol, System};
use crate::models::{Register201ResponseData, RegisterRequest};
use log::debug;

use crate::rate_limiter::PriorityRateLimiter;

#[derive(Debug, Clone)]
pub struct Api {
    configuration: Arc<Configuration>,
    limiter: Arc<PriorityRateLimiter>,
}

#[allow(dead_code)]
impl Api {
    pub fn new(access_token: Option<String>, quota: u64, burst: NonZeroU32) -> Api {
        // Create a rate limiter: 2 requests per 1 seconds
        // let quota = Quota::with_period(Duration::from_millis(550)).unwrap();

        Api {
            configuration: Arc::new(Configuration {
                bearer_access_token: access_token,
                ..Default::default()
            }),
            limiter: Arc::new(PriorityRateLimiter::new(quota, burst)),
        }
    }

    pub fn get_configuration(&self) -> Arc<Configuration> {
        self.configuration.clone()
    }

    pub fn get_limiter(&self) -> Arc<PriorityRateLimiter> {
        self.limiter.clone()
    }

    /// Return the status of the game server. This also includes a few global elements, such as announcements, server reset dates and leaderboards.
    pub async fn get_status(&self) -> Result<models::GetStatus200Response, Error<GetStatusError>> {
        self.limiter.until_ready(50, "").await;
        let result = crate::apis::global_api::get_status(&self.configuration).await?;

        Ok(result)
    }

    /// Creates a new agent and ties it to an account.  The agent symbol must consist of a 3-14 character string, and will be used to represent your agent. This symbol will prefix the symbol of every ship you own. Agent symbols will be cast to all uppercase characters.  This new agent will be tied to a starting faction of your choice, which determines your starting location, and will be granted an authorization token, a contract with their starting faction, a command ship that can fly across space with advanced capabilities, a small probe ship that can be used for reconnaissance, and 150,000 credits.  > #### Keep your token safe and secure > > Save your token during the alpha phase. There is no way to regenerate this token without starting a new agent. In the future you will be able to generate and manage your tokens from the SpaceTraders website.  If you are new to SpaceTraders, It is recommended to register with the COSMIC faction, a faction that is well connected to the rest of the universe. After registering, you should try our interactive [quickstart guide](https://docs.spacetraders.io/quickstart/new-game) which will walk you through basic API requests in just a few minutes.
    pub async fn register(
        &self,
        symbol: String,
        faction: FactionSymbol,
        account_token: String,
    ) -> Result<Register201ResponseData, Error<RegisterError>> {
        let mut config = Configuration::clone(&self.configuration);
        config.bearer_access_token = Some(account_token);
        let register_response: models::Register201Response =
            crate::apis::accounts_api::register(&config, RegisterRequest { symbol, faction })
                .await?;

        Ok(*register_response.data)
    }

    /// Fetch your agent's details.
    pub async fn get_my_account(
        &self,
        account_token: String,
    ) -> Result<models::GetMyAccount200ResponseData, Error<GetMyAccountError>> {
        let mut config = Configuration::clone(&self.configuration);
        config.bearer_access_token = Some(account_token);
        let register_response = crate::apis::accounts_api::get_my_account(&config).await?;

        Ok(*register_response.data)
    }

    /// Fetch agent details.
    pub async fn get_agent(
        &self,
        agent_symbol: &str,
    ) -> Result<models::GetAgent200Response, Error<GetAgentError>> {
        self.limiter.until_ready(50, "get_agent").await;
        let result = crate::apis::agents_api::get_agent(&self.configuration, agent_symbol).await?;

        Ok(result)
    }

    /// Fetch agents details.
    pub async fn get_agents(
        &self,
        page: Option<i32>,
        limit: Option<i32>,
    ) -> Result<models::GetAgents200Response, Error<GetAgentsError>> {
        self.limiter.until_ready(50, "get_agents").await;
        let result = crate::apis::agents_api::get_agents(&self.configuration, page, limit).await?;

        Ok(result)
    }

    pub async fn get_all_agents(
        &self,
        limit: i32,
    ) -> Result<Vec<models::PublicAgent>, Error<GetAgentsError>> {
        if !(1..=20).contains(&limit) {
            panic!("Invalid limit must be between 1 and 20");
        }
        let mut current_page = 1;

        let mut agents: Vec<models::PublicAgent> = Vec::new();

        loop {
            let page = self.get_agents(Some(current_page), Some(limit)).await?;
            agents.extend(page.data);
            let total = page.meta.total;

            debug!(
                "limit: {}, page {} of {}, agents: {} of {}",
                limit,
                current_page,
                total / limit + 1,
                agents.len(),
                total
            );

            if agents.len() >= total.try_into().unwrap() {
                break;
            }

            current_page += 1;
        }
        Ok(agents)
    }

    /// Fetch your agent's details.
    pub async fn get_my_agent(
        &self,
    ) -> Result<models::GetMyAgent200Response, Error<GetMyAgentError>> {
        self.limiter.until_ready(50, "get_my_agent").await;
        let result = crate::apis::agents_api::get_my_agent(&self.configuration).await?;

        Ok(result)
    }

    /// Get recent events for your agent.
    pub async fn get_my_agent_events(
        &self,
    ) -> Result<models::GetMyAgentEvents200Response, Error<GetMyAgentEventsError>> {
        self.limiter.until_ready(50, "get_my_agent_events").await;
        let result = crate::apis::agents_api::get_my_agent_events(&self.configuration).await?;

        Ok(result)
    }

    /// Accept a contract by ID.   You can only accept contracts that were offered to you, were not accepted yet, and whose deadlines has not passed yet.
    pub async fn accept_contract(
        &self,
        contract_id: &str,
    ) -> Result<models::AcceptContract200Response, Error<AcceptContractError>> {
        if self.configuration.bearer_access_token.is_none() {
            panic!("Invalid bearer_access_token");
        }
        self.limiter.until_ready(50, "").await;
        let result =
            crate::apis::contracts_api::accept_contract(&self.configuration, contract_id).await?;

        Ok(result)
    }

    /// Deliver cargo to a contract.  In order to use this API, a ship must be at the delivery location (denoted in the delivery terms as `destinationSymbol` of a contract) and must have a number of units of a good required by this contract in its cargo.  Cargo that was delivered will be removed from the ship's cargo.
    pub async fn deliver_contract(
        &self,
        contract_id: &str,
        deliver_contract_request: models::DeliverContractRequest,
    ) -> Result<models::DeliverContract200Response, Error<DeliverContractError>> {
        if self.configuration.bearer_access_token.is_none() {
            panic!("Invalid bearer_access_token");
        }
        self.limiter.until_ready(50, "deliver_contract").await;
        let result = crate::apis::contracts_api::deliver_contract(
            &self.configuration,
            contract_id,
            deliver_contract_request,
        )
        .await?;

        Ok(result)
    }

    /// Fulfill a contract. Can only be used on contracts that have all of their delivery terms fulfilled.
    pub async fn fulfill_contract(
        &self,
        contract_id: &str,
    ) -> Result<models::FulfillContract200Response, Error<FulfillContractError>> {
        if self.configuration.bearer_access_token.is_none() {
            panic!("Invalid bearer_access_token");
        }
        self.limiter.until_ready(50, "fulfill_contract").await;
        let result =
            crate::apis::contracts_api::fulfill_contract(&self.configuration, contract_id).await?;

        Ok(result)
    }

    /// Get the details of a contract by ID.
    pub async fn get_contract(
        &self,
        contract_id: &str,
    ) -> Result<models::GetContract200Response, Error<GetContractError>> {
        if self.configuration.bearer_access_token.is_none() {
            panic!("Invalid bearer_access_token");
        }
        self.limiter.until_ready(50, "get_contract").await;
        let result =
            crate::apis::contracts_api::get_contract(&self.configuration, contract_id).await?;

        Ok(result)
    }

    /// Return a paginated list of all your contracts.
    pub async fn get_contracts(
        &self,
        page: Option<i32>,
        limit: Option<i32>,
    ) -> Result<models::GetContracts200Response, Error<GetContractsError>> {
        if self.configuration.bearer_access_token.is_none() {
            panic!("Invalid bearer_access_token");
        }
        self.limiter.until_ready(50, "get_contracts").await;
        let result =
            crate::apis::contracts_api::get_contracts(&self.configuration, page, limit).await?;

        Ok(result)
    }

    /// Returns ALL your contracts.
    pub async fn get_all_contracts(
        &self,
        limit: i32,
    ) -> Result<Vec<models::Contract>, Error<GetContractsError>> {
        if self.configuration.bearer_access_token.is_none() {
            panic!("Invalid bearer_access_token");
        }
        if !(1..=20).contains(&limit) {
            panic!("Invalid limit must be between 1 and 20");
        }
        let mut current_page = 1;

        let mut contracts: Vec<models::Contract> = Vec::new();

        loop {
            let page = self.get_contracts(Some(current_page), Some(limit)).await?;
            contracts.extend(page.data);
            let total = page.meta.total;

            debug!(
                "limit: {}, page {} of {}, contracts: {} of {}",
                limit,
                current_page,
                total / limit + 1,
                contracts.len(),
                total
            );

            if contracts.len() >= total.try_into().unwrap() {
                break;
            }

            current_page += 1;
        }
        Ok(contracts)
    }

    /// View the details of a faction.
    pub async fn get_faction(
        &self,
        faction_symbol: &str,
    ) -> Result<models::GetFaction200Response, Error<GetFactionError>> {
        self.limiter.until_ready(50, "get_faction").await;
        let result =
            crate::apis::factions_api::get_faction(&self.configuration, faction_symbol).await?;

        Ok(result)
    }

    /// Return a paginated list of all the factions in the game.
    pub async fn get_factions(
        &self,
        page: Option<i32>,
        limit: Option<i32>,
    ) -> Result<models::GetFactions200Response, Error<GetFactionsError>> {
        self.limiter.until_ready(50, "get_factions").await;
        let result =
            crate::apis::factions_api::get_factions(&self.configuration, page, limit).await?;

        Ok(result)
    }

    pub async fn get_all_factions(
        &self,
        limit: i32,
    ) -> Result<Vec<models::Faction>, Error<GetFactionsError>> {
        if !(1..=20).contains(&limit) {
            panic!("Invalid limit must be between 1 and 20");
        }
        let mut current_page = 1;

        let mut factions: Vec<models::Faction> = Vec::new();

        loop {
            let page = self.get_factions(Some(current_page), Some(limit)).await?;
            factions.extend(page.data);
            let total = page.meta.total;

            debug!(
                "limit: {}, page {} of {}, factions: {} of {}",
                limit,
                current_page,
                total / limit + 1,
                factions.len(),
                total
            );

            if factions.len() >= total.try_into().unwrap() {
                break;
            }

            current_page += 1;
        }
        Ok(factions)
    }

    /// Command a ship to chart the waypoint at its current location.  Most waypoints in the universe are uncharted by default. These waypoints have their traits hidden until they have been charted by a ship.  Charting a waypoint will record your agent as the one who created the chart, and all other agents would also be able to see the waypoint's traits.
    pub async fn create_chart(
        &self,
        ship_symbol: &str,
    ) -> Result<models::CreateChart201Response, Error<CreateChartError>> {
        if self.configuration.bearer_access_token.is_none() {
            panic!("Invalid bearer_access_token");
        }
        self.limiter.until_ready(50, "create_chart").await;
        let result = crate::apis::fleet_api::create_chart(&self.configuration, ship_symbol).await?;

        Ok(result)
    }

    /// Scan for nearby ships, retrieving information for all ships in range.  Requires a ship to have the `Sensor Array` mount installed to use.  The ship will enter a cooldown after using this function, during which it cannot execute certain actions.
    pub async fn create_ship_ship_scan(
        &self,
        ship_symbol: &str,
    ) -> Result<models::CreateShipShipScan201Response, Error<CreateShipShipScanError>> {
        if self.configuration.bearer_access_token.is_none() {
            panic!("Invalid bearer_access_token");
        }
        self.limiter.until_ready(50, "create_ship_ship_scan").await;
        let result =
            crate::apis::fleet_api::create_ship_ship_scan(&self.configuration, ship_symbol).await?;

        Ok(result)
    }

    /// Scan for nearby systems, retrieving information on the systems' distance from the ship and their waypoints. Requires a ship to have the `Sensor Array` mount installed to use.  The ship will enter a cooldown after using this function, during which it cannot execute certain actions.
    pub async fn create_ship_system_scan(
        &self,
        ship_symbol: &str,
    ) -> Result<models::CreateShipSystemScan201Response, Error<CreateShipSystemScanError>> {
        if self.configuration.bearer_access_token.is_none() {
            panic!("Invalid bearer_access_token");
        }
        self.limiter
            .until_ready(50, "create_ship_system_scan")
            .await;
        let result =
            crate::apis::fleet_api::create_ship_system_scan(&self.configuration, ship_symbol)
                .await?;

        Ok(result)
    }

    /// Scan for nearby waypoints, retrieving detailed information on each waypoint in range. Scanning uncharted waypoints will allow you to ignore their uncharted state and will list the waypoints' traits.  Requires a ship to have the `Sensor Array` mount installed to use.  The ship will enter a cooldown after using this function, during which it cannot execute certain actions.
    pub async fn create_ship_waypoint_scan(
        &self,
        ship_symbol: &str,
    ) -> Result<models::CreateShipWaypointScan201Response, Error<CreateShipWaypointScanError>> {
        if self.configuration.bearer_access_token.is_none() {
            panic!("Invalid bearer_access_token");
        }
        self.limiter
            .until_ready(50, "create_ship_waypoint_scan")
            .await;
        let result =
            crate::apis::fleet_api::create_ship_waypoint_scan(&self.configuration, ship_symbol)
                .await?;

        Ok(result)
    }

    /// Create surveys on a waypoint that can be extracted such as asteroid fields. A survey focuses on specific types of deposits from the extracted location. When ships extract using this survey, they are guaranteed to procure a high amount of one of the goods in the survey.  In order to use a survey, send the entire survey details in the body of the extract request.  Each survey may have multiple deposits, and if a symbol shows up more than once, that indicates a higher chance of extracting that resource.  Your ship will enter a cooldown after surveying in which it is unable to perform certain actions. Surveys will eventually expire after a period of time or will be exhausted after being extracted several times based on the survey's size. Multiple ships can use the same survey for extraction.  A ship must have the `Surveyor` mount installed in order to use this function.
    pub async fn create_survey(
        &self,
        ship_symbol: &str,
    ) -> Result<models::CreateSurvey201Response, Error<CreateSurveyError>> {
        if self.configuration.bearer_access_token.is_none() {
            panic!("Invalid bearer_access_token");
        }
        self.limiter.until_ready(50, "create_survey").await;
        let result =
            crate::apis::fleet_api::create_survey(&self.configuration, ship_symbol).await?;

        Ok(result)
    }

    /// Attempt to dock your ship at its current location. Docking will only succeed if your ship is capable of docking at the time of the request.  Docked ships can access elements in their current location, such as the market or a shipyard, but cannot do actions that require the ship to be above surface such as navigating or extracting.  The endpoint is idempotent - successive calls will succeed even if the ship is already docked.
    pub async fn dock_ship(
        &self,
        ship_symbol: &str,
    ) -> Result<models::DockShip200Response, Error<DockShipError>> {
        if self.configuration.bearer_access_token.is_none() {
            panic!("Invalid bearer_access_token");
        }
        self.limiter.until_ready(50, "dock_ship").await;
        let result = crate::apis::fleet_api::dock_ship(&self.configuration, ship_symbol).await?;

        Ok(result)
    }

    /// Extract resources from a waypoint that can be extracted, such as asteroid fields, into your ship. Send an optional survey as the payload to target specific yields.  The ship must be in orbit to be able to extract and must have mining equipments installed that can extract goods, such as the `Gas Siphon` mount for gas-based goods or `Mining Laser` mount for ore-based goods.  The survey property is now deprecated. See the `extract/survey` endpoint for more details.
    pub async fn extract_resources(
        &self,
        ship_symbol: &str,
    ) -> Result<models::ExtractResources201Response, Error<ExtractResourcesError>> {
        if self.configuration.bearer_access_token.is_none() {
            panic!("Invalid bearer_access_token");
        }
        self.limiter.until_ready(50, "extract_resources").await;
        let result =
            crate::apis::fleet_api::extract_resources(&self.configuration, ship_symbol).await?;

        Ok(result)
    }

    /// Use a survey when extracting resources from a waypoint. This endpoint requires a survey as the payload, which allows your ship to extract specific yields.  Send the full survey object as the payload which will be validated according to the signature. If the signature is invalid, or any properties of the survey are changed, the request will fail.
    pub async fn extract_resources_with_survey(
        &self,
        ship_symbol: &str,
        survey: Option<models::Survey>,
    ) -> Result<models::ExtractResources201Response, Error<ExtractResourcesWithSurveyError>> {
        if self.configuration.bearer_access_token.is_none() {
            panic!("Invalid bearer_access_token");
        }
        self.limiter
            .until_ready(50, "extract_resources_with_survey")
            .await;
        let result = crate::apis::fleet_api::extract_resources_with_survey(
            &self.configuration,
            ship_symbol,
            survey,
        )
        .await?;

        Ok(result)
    }

    /// Get the mounts installed on a ship.
    pub async fn get_mounts(
        &self,
        ship_symbol: &str,
    ) -> Result<models::GetMounts200Response, Error<GetMountsError>> {
        if self.configuration.bearer_access_token.is_none() {
            panic!("Invalid bearer_access_token");
        }
        self.limiter.until_ready(50, "get_mounts").await;
        let result = crate::apis::fleet_api::get_mounts(&self.configuration, ship_symbol).await?;

        Ok(result)
    }

    /// Get the modules installed on a ship.
    pub async fn get_ship_modules(
        &self,
        ship_symbol: &str,
    ) -> Result<models::GetShipModules200Response, Error<GetShipModulesError>> {
        if self.configuration.bearer_access_token.is_none() {
            panic!("Invalid bearer_access_token");
        }
        self.limiter.until_ready(50, "get_ship_modules").await;
        let result =
            crate::apis::fleet_api::get_ship_modules(&self.configuration, ship_symbol).await?;

        Ok(result)
    }

    /// Retrieve the details of a ship under your agent's ownership.
    pub async fn get_my_ship(
        &self,
        ship_symbol: &str,
    ) -> Result<models::GetMyShip200Response, Error<GetMyShipError>> {
        if self.configuration.bearer_access_token.is_none() {
            panic!("Invalid bearer_access_token");
        }
        self.limiter.until_ready(50, "get_my_ship").await;
        let result = crate::apis::fleet_api::get_my_ship(&self.configuration, ship_symbol).await?;

        Ok(result)
    }

    /// Retrieve the cargo of a ship under your agent's ownership.
    pub async fn get_my_ship_cargo(
        &self,
        ship_symbol: &str,
    ) -> Result<models::GetMyShipCargo200Response, Error<GetMyShipCargoError>> {
        if self.configuration.bearer_access_token.is_none() {
            panic!("Invalid bearer_access_token");
        }
        self.limiter.until_ready(50, "get_my_ship_cargo").await;
        let result =
            crate::apis::fleet_api::get_my_ship_cargo(&self.configuration, ship_symbol).await?;

        Ok(result)
    }

    /// Return a paginated list of all of ships under your agent's ownership.
    pub async fn get_my_ships(
        &self,
        page: Option<i32>,
        limit: Option<i32>,
    ) -> Result<models::GetMyShips200Response, Error<GetMyShipsError>> {
        if self.configuration.bearer_access_token.is_none() {
            panic!("Invalid bearer_access_token");
        }
        self.limiter.until_ready(50, "get_my_ships").await;
        let result = crate::apis::fleet_api::get_my_ships(&self.configuration, page, limit).await?;

        Ok(result)
    }

    /// Returns ALL ships under your agent's ownership.
    pub async fn get_all_my_ships(
        &self,
        limit: i32,
    ) -> Result<Vec<models::Ship>, Error<GetMyShipsError>> {
        if self.configuration.bearer_access_token.is_none() {
            panic!("Invalid bearer_access_token");
        }
        if !(1..=20).contains(&limit) {
            panic!("Invalid limit must be between 1 and 20");
        }
        let mut current_page = 1;

        let mut ships: Vec<models::Ship> = Vec::new();

        loop {
            let page = self.get_my_ships(Some(current_page), Some(limit)).await?;
            ships.extend(page.data);
            let total = page.meta.total;

            debug!(
                "limit: {}, page {} of {}, ships: {} of {}",
                limit,
                current_page,
                total / limit + 1,
                ships.len(),
                total
            );

            if ships.len() >= total.try_into().unwrap() {
                break;
            }

            current_page += 1;
        }
        Ok(ships)
    }

    /// Get the cost of repairing a ship.
    pub async fn get_repair_ship(
        &self,
        ship_symbol: &str,
    ) -> Result<models::GetRepairShip200Response, Error<GetRepairShipError>> {
        if self.configuration.bearer_access_token.is_none() {
            panic!("Invalid bearer_access_token");
        }
        self.limiter.until_ready(50, "get_repair_ship").await;
        let result =
            crate::apis::fleet_api::get_repair_ship(&self.configuration, ship_symbol).await?;

        Ok(result)
    }

    /// Get the amount of value that will be returned when scrapping a ship.
    pub async fn get_scrap_ship(
        &self,
        ship_symbol: &str,
    ) -> Result<models::GetScrapShip200Response, Error<GetScrapShipError>> {
        if self.configuration.bearer_access_token.is_none() {
            panic!("Invalid bearer_access_token");
        }
        self.limiter.until_ready(50, "get_scrap_ship").await;
        let result =
            crate::apis::fleet_api::get_scrap_ship(&self.configuration, ship_symbol).await?;

        Ok(result)
    }

    /// Retrieve the details of your ship's reactor cooldown. Some actions such as activating your jump drive, scanning, or extracting resources taxes your reactor and results in a cooldown.  Your ship cannot perform additional actions until your cooldown has expired. The duration of your cooldown is relative to the power consumption of the related modules or mounts for the action taken.  Response returns a 204 status code (no-content) when the ship has no cooldown.
    pub async fn get_ship_cooldown(
        &self,
        ship_symbol: &str,
    ) -> Result<models::GetShipCooldown200Response, Error<GetShipCooldownError>> {
        if self.configuration.bearer_access_token.is_none() {
            panic!("Invalid bearer_access_token");
        }
        self.limiter.until_ready(50, "get_ship_cooldown").await;
        let result =
            crate::apis::fleet_api::get_ship_cooldown(&self.configuration, ship_symbol).await?;

        Ok(result)
    }

    /// Get the current nav status of a ship.
    pub async fn get_ship_nav(
        &self,
        ship_symbol: &str,
    ) -> Result<models::GetShipNav200Response, Error<GetShipNavError>> {
        if self.configuration.bearer_access_token.is_none() {
            panic!("Invalid bearer_access_token");
        }
        self.limiter.until_ready(50, "get_ship_nav").await;
        let result = crate::apis::fleet_api::get_ship_nav(&self.configuration, ship_symbol).await?;

        Ok(result)
    }

    /// Install a mount on a ship.  In order to install a mount, the ship must be docked and located in a waypoint that has a `Shipyard` trait. The ship also must have the mount to install in its cargo hold.  An installation fee will be deduced by the Shipyard for installing the mount on the ship.
    pub async fn install_mount(
        &self,
        ship_symbol: &str,
        install_mount_request: models::InstallMountRequest,
    ) -> Result<models::InstallMount201Response, Error<InstallMountError>> {
        if self.configuration.bearer_access_token.is_none() {
            panic!("Invalid bearer_access_token");
        }
        self.limiter.until_ready(50, "install_mount").await;
        let result = crate::apis::fleet_api::install_mount(
            &self.configuration,
            ship_symbol,
            install_mount_request,
        )
        .await?;

        Ok(result)
    }

    /// Install a module on a ship. The module must be in your cargo.
    pub async fn install_ship_module(
        &self,
        ship_symbol: &str,
        install_ship_module_request: models::InstallShipModuleRequest,
    ) -> Result<models::InstallShipModule201Response, Error<InstallShipModuleError>> {
        if self.configuration.bearer_access_token.is_none() {
            panic!("Invalid bearer_access_token");
        }
        self.limiter.until_ready(50, "install_ship_module").await;
        let result = crate::apis::fleet_api::install_ship_module(
            &self.configuration,
            ship_symbol,
            install_ship_module_request,
        )
        .await?;

        Ok(result)
    }

    /// Jettison cargo from your ship's cargo hold.
    pub async fn jettison(
        &self,
        ship_symbol: &str,
        jettison_request: models::JettisonRequest,
    ) -> Result<models::Jettison200Response, Error<JettisonError>> {
        if self.configuration.bearer_access_token.is_none() {
            panic!("Invalid bearer_access_token");
        }
        self.limiter.until_ready(50, "jettison").await;
        let result =
            crate::apis::fleet_api::jettison(&self.configuration, ship_symbol, jettison_request)
                .await?;

        Ok(result)
    }

    /// Jump your ship instantly to a target connected waypoint. The ship must be in orbit to execute a jump.  A unit of antimatter is purchased and consumed from the market when jumping. The price of antimatter is determined by the market and is subject to change. A ship can only jump to connected waypoints
    pub async fn jump_ship(
        &self,
        ship_symbol: &str,
        jump_ship_request: models::JumpShipRequest,
    ) -> Result<models::JumpShip200Response, Error<JumpShipError>> {
        if self.configuration.bearer_access_token.is_none() {
            panic!("Invalid bearer_access_token");
        }
        self.limiter.until_ready(50, "jump_ship").await;
        let result =
            crate::apis::fleet_api::jump_ship(&self.configuration, ship_symbol, jump_ship_request)
                .await?;

        Ok(result)
    }

    /// Navigate to a target destination. The ship must be in orbit to use this function. The destination waypoint must be within the same system as the ship's current location. Navigating will consume the necessary fuel from the ship's manifest based on the distance to the target waypoint.  The returned response will detail the route information including the expected time of arrival. Most ship actions are unavailable until the ship has arrived at it's destination.  To travel between systems, see the ship's Warp or Jump actions.
    pub async fn navigate_ship(
        &self,
        ship_symbol: &str,
        navigate_ship_request: models::NavigateShipRequest,
    ) -> Result<models::NavigateShip200Response, Error<NavigateShipError>> {
        if self.configuration.bearer_access_token.is_none() {
            panic!("Invalid bearer_access_token");
        }
        self.limiter.until_ready(50, "navigate_ship").await;
        let result = crate::apis::fleet_api::navigate_ship(
            &self.configuration,
            ship_symbol,
            navigate_ship_request,
        )
        .await?;

        Ok(result)
    }

    /// Negotiate a new contract with the HQ.  In order to negotiate a new contract, an agent must not have ongoing or offered contracts over the allowed maximum amount. Currently the maximum contracts an agent can have at a time is 1.  Once a contract is negotiated, it is added to the list of contracts offered to the agent, which the agent can then accept.   The ship must be present at any waypoint with a faction present to negotiate a contract with that faction.
    pub async fn negotiate_contract(
        &self,
        ship_symbol: &str,
    ) -> Result<
        models::NegotiateContract201Response,
        Error<crate::apis::contracts_api::NegotiateContractError>,
    > {
        if self.configuration.bearer_access_token.is_none() {
            panic!("Invalid bearer_access_token");
        }
        self.limiter.until_ready(50, "negotiate_contract").await;
        // let result =
        //     crate::apis::fleet_api::negotiate_contract(&self.configuration, ship_symbol).await?;

        let result =
            crate::apis::contracts_api::negotiate_contract(&self.configuration, ship_symbol)
                .await?;

        Ok(result)
    }

    /// Attempt to move your ship into orbit at its current location. The request will only succeed if your ship is capable of moving into orbit at the time of the request.  Orbiting ships are able to do actions that require the ship to be above surface such as navigating or extracting, but cannot access elements in their current waypoint, such as the market or a shipyard.  The endpoint is idempotent - successive calls will succeed even if the ship is already in orbit.
    pub async fn orbit_ship(
        &self,
        ship_symbol: &str,
    ) -> Result<models::OrbitShip200Response, Error<OrbitShipError>> {
        if self.configuration.bearer_access_token.is_none() {
            panic!("Invalid bearer_access_token");
        }
        self.limiter.until_ready(50, "orbit_ship").await;
        let result = crate::apis::fleet_api::orbit_ship(&self.configuration, ship_symbol).await?;

        Ok(result)
    }

    /// Update the nav configuration of a ship.  Currently only supports configuring the Flight Mode of the ship, which affects its speed and fuel consumption.
    pub async fn patch_ship_nav(
        &self,
        ship_symbol: &str,
        patch_ship_nav_request: Option<models::PatchShipNavRequest>,
    ) -> Result<models::PatchShipNav200Response, Error<PatchShipNavError>> {
        if self.configuration.bearer_access_token.is_none() {
            panic!("Invalid bearer_access_token");
        }
        self.limiter.until_ready(50, "patch_ship_nav").await;
        let result = crate::apis::fleet_api::patch_ship_nav(
            &self.configuration,
            ship_symbol,
            patch_ship_nav_request,
        )
        .await?;

        Ok(result)
    }

    /// Purchase cargo from a market.  The ship must be docked in a waypoint that has `Marketplace` trait, and the market must be selling a good to be able to purchase it.  The maximum amount of units of a good that can be purchased in each transaction are denoted by the `tradeVolume` value of the good, which can be viewed by using the Get Market action.  Purchased goods are added to the ship's cargo hold.
    pub async fn purchase_cargo(
        &self,
        ship_symbol: &str,
        purchase_cargo_request: models::PurchaseCargoRequest,
    ) -> Result<models::PurchaseCargo201Response, Error<PurchaseCargoError>> {
        if self.configuration.bearer_access_token.is_none() {
            panic!("Invalid bearer_access_token");
        }
        self.limiter.until_ready(50, "purchase_cargo").await;
        let result = crate::apis::fleet_api::purchase_cargo(
            &self.configuration,
            ship_symbol,
            purchase_cargo_request,
        )
        .await?;

        Ok(result)
    }

    /// Purchase a ship from a Shipyard. In order to use this function, a ship under your agent's ownership must be in a waypoint that has the `Shipyard` trait, and the Shipyard must sell the type of the desired ship.  Shipyards typically offer ship types, which are predefined templates of ships that have dedicated roles. A template comes with a preset of an engine, a reactor, and a frame. It may also include a few modules and mounts.
    pub async fn purchase_ship(
        &self,
        purchase_ship_request: models::PurchaseShipRequest,
    ) -> Result<models::PurchaseShip201Response, Error<PurchaseShipError>> {
        if self.configuration.bearer_access_token.is_none() {
            panic!("Invalid bearer_access_token");
        }
        self.limiter.until_ready(50, "purchase_ship").await;
        let result =
            crate::apis::fleet_api::purchase_ship(&self.configuration, purchase_ship_request)
                .await?;

        Ok(result)
    }

    /// Refuel your ship by buying fuel from the local market.  Requires the ship to be docked in a waypoint that has the `Marketplace` trait, and the market must be selling fuel in order to refuel.  Each fuel bought from the market replenishes 100 units in your ship's fuel.  Ships will always be refuel to their frame's maximum fuel capacity when using this action.
    pub async fn refuel_ship(
        &self,
        ship_symbol: &str,
        refuel_ship_request: Option<models::RefuelShipRequest>,
    ) -> Result<models::RefuelShip200Response, Error<RefuelShipError>> {
        if self.configuration.bearer_access_token.is_none() {
            panic!("Invalid bearer_access_token");
        }
        self.limiter.until_ready(50, "refuel_ship").await;
        let result = crate::apis::fleet_api::refuel_ship(
            &self.configuration,
            ship_symbol,
            refuel_ship_request,
        )
        .await?;

        Ok(result)
    }

    /// Remove a mount from a ship.  The ship must be docked in a waypoint that has the `Shipyard` trait, and must have the desired mount that it wish to remove installed.  A removal fee will be deduced from the agent by the Shipyard.
    pub async fn remove_mount(
        &self,
        ship_symbol: &str,
        remove_mount_request: models::RemoveMountRequest,
    ) -> Result<models::RemoveMount201Response, Error<RemoveMountError>> {
        if self.configuration.bearer_access_token.is_none() {
            panic!("Invalid bearer_access_token");
        }
        self.limiter.until_ready(50, "remove_mount").await;
        let result = crate::apis::fleet_api::remove_mount(
            &self.configuration,
            ship_symbol,
            remove_mount_request,
        )
        .await?;

        Ok(result)
    }

    /// Remove a module from a ship. The module will be placed in cargo.
    pub async fn remove_ship_module(
        &self,
        ship_symbol: &str,
        remove_ship_module_request: models::RemoveShipModuleRequest,
    ) -> Result<models::RemoveShipModule201Response, Error<RemoveShipModuleError>> {
        if self.configuration.bearer_access_token.is_none() {
            panic!("Invalid bearer_access_token");
        }
        self.limiter.until_ready(50, "remove_ship_module").await;
        let result = crate::apis::fleet_api::remove_ship_module(
            &self.configuration,
            ship_symbol,
            remove_ship_module_request,
        )
        .await?;

        Ok(result)
    }

    /// Repair a ship, restoring the ship to maximum condition. The ship must be docked at a waypoint that has the `Shipyard` trait in order to use this function. To preview the cost of repairing the ship, use the Get action.
    pub async fn repair_ship(
        &self,
        ship_symbol: &str,
    ) -> Result<models::RepairShip200Response, Error<RepairShipError>> {
        if self.configuration.bearer_access_token.is_none() {
            panic!("Invalid bearer_access_token");
        }
        self.limiter.until_ready(50, "repair_ship").await;
        let result = crate::apis::fleet_api::repair_ship(&self.configuration, ship_symbol).await?;

        Ok(result)
    }

    /// Scrap a ship, removing it from the game and returning a portion of the ship's value to the agent. The ship must be docked in a waypoint that has the `Shipyard` trait in order to use this function. To preview the amount of value that will be returned, use the Get Ship action.
    pub async fn scrap_ship(
        &self,
        ship_symbol: &str,
    ) -> Result<models::ScrapShip200Response, Error<ScrapShipError>> {
        if self.configuration.bearer_access_token.is_none() {
            panic!("Invalid bearer_access_token");
        }
        self.limiter.until_ready(50, "scrap_ship").await;
        let result = crate::apis::fleet_api::scrap_ship(&self.configuration, ship_symbol).await?;

        Ok(result)
    }

    /// Sell cargo in your ship to a market that trades this cargo. The ship must be docked in a waypoint that has the `Marketplace` trait in order to use this function.
    pub async fn sell_cargo(
        &self,
        ship_symbol: &str,
        sell_cargo_request: models::SellCargoRequest,
    ) -> Result<models::SellCargo201Response, Error<SellCargoError>> {
        if self.configuration.bearer_access_token.is_none() {
            panic!("Invalid bearer_access_token");
        }
        self.limiter.until_ready(50, "sell_cargo").await;
        let result = crate::apis::fleet_api::sell_cargo(
            &self.configuration,
            ship_symbol,
            sell_cargo_request,
        )
        .await?;

        Ok(result)
    }

    /// Attempt to refine the raw materials on your ship. The request will only succeed if your ship is capable of refining at the time of the request. In order to be able to refine, a ship must have goods that can be refined and have installed a `Refinery` module that can refine it.  When refining, 30 basic goods will be converted into 10 processed goods.
    pub async fn ship_refine(
        &self,
        ship_symbol: &str,
        ship_refine_request: models::ShipRefineRequest,
    ) -> Result<models::ShipRefine201Response, Error<ShipRefineError>> {
        if self.configuration.bearer_access_token.is_none() {
            panic!("Invalid bearer_access_token");
        }
        self.limiter.until_ready(50, "ship_refine").await;
        let result = crate::apis::fleet_api::ship_refine(
            &self.configuration,
            ship_symbol,
            ship_refine_request,
        )
        .await?;

        Ok(result)
    }

    /// Siphon gases, such as hydrocarbon, from gas giants.  The ship must be in orbit to be able to siphon and must have siphon mounts and a gas processor installed.
    pub async fn siphon_resources(
        &self,
        ship_symbol: &str,
    ) -> Result<models::SiphonResources201Response, Error<SiphonResourcesError>> {
        if self.configuration.bearer_access_token.is_none() {
            panic!("Invalid bearer_access_token");
        }
        self.limiter.until_ready(50, "siphon_resources").await;
        let result =
            crate::apis::fleet_api::siphon_resources(&self.configuration, ship_symbol).await?;

        Ok(result)
    }

    /// Transfer cargo between ships.  The receiving ship must be in the same waypoint as the transferring ship, and it must able to hold the additional cargo after the transfer is complete. Both ships also must be in the same state, either both are docked or both are orbiting.  The response body's cargo shows the cargo of the transferring ship after the transfer is complete.
    pub async fn transfer_cargo(
        &self,
        ship_symbol: &str,
        transfer_cargo_request: models::TransferCargoRequest,
    ) -> Result<models::TransferCargo200Response, Error<TransferCargoError>> {
        if self.configuration.bearer_access_token.is_none() {
            panic!("Invalid bearer_access_token");
        }
        self.limiter.until_ready(50, "transfer_cargo").await;
        let result = crate::apis::fleet_api::transfer_cargo(
            &self.configuration,
            ship_symbol,
            transfer_cargo_request,
        )
        .await?;

        Ok(result)
    }

    /// Warp your ship to a target destination in another system. The ship must be in orbit to use this function and must have the `Warp Drive` module installed. Warping will consume the necessary fuel from the ship's manifest.  The returned response will detail the route information including the expected time of arrival. Most ship actions are unavailable until the ship has arrived at its destination.
    pub async fn warp_ship(
        &self,
        ship_symbol: &str,
        navigate_ship_request: models::NavigateShipRequest,
    ) -> Result<models::NavigateShip200Response, Error<WarpShipError>> {
        if self.configuration.bearer_access_token.is_none() {
            panic!("Invalid bearer_access_token");
        }
        self.limiter.until_ready(50, "warp_ship").await;
        let result = crate::apis::fleet_api::warp_ship(
            &self.configuration,
            ship_symbol,
            navigate_ship_request,
        )
        .await?;

        Ok(result)
    }

    /// Get construction details for a waypoint. Requires a waypoint with a property of `isUnderConstruction` to be true.
    pub async fn get_construction(
        &self,
        system_symbol: &str,
        waypoint_symbol: &str,
    ) -> Result<models::GetConstruction200Response, Error<GetConstructionError>> {
        self.limiter.until_ready(50, "get_construction").await;
        let result = crate::apis::systems_api::get_construction(
            &self.configuration,
            system_symbol,
            waypoint_symbol,
        )
        .await?;

        Ok(result)
    }

    /// Get jump gate details for a waypoint. Requires a waypoint of type `JUMP_GATE` to use.  Waypoints connected to this jump gate can be
    pub async fn get_jump_gate(
        &self,
        system_symbol: &str,
        waypoint_symbol: &str,
    ) -> Result<models::GetJumpGate200Response, Error<GetJumpGateError>> {
        self.limiter.until_ready(50, "get_jump_gate").await;
        let result = crate::apis::systems_api::get_jump_gate(
            &self.configuration,
            system_symbol,
            waypoint_symbol,
        )
        .await?;

        Ok(result)
    }

    /// Retrieve imports, exports and exchange data from a marketplace. Requires a waypoint that has the `Marketplace` trait to use.  Send a ship to the waypoint to access trade good prices and recent transactions. Refer to the [Market Overview page](https://docs.spacetraders.io/game-concepts/markets) to gain better a understanding of the market in the game.
    pub async fn get_market(
        &self,
        system_symbol: &str,
        waypoint_symbol: &str,
    ) -> Result<models::GetMarket200Response, Error<GetMarketError>> {
        self.limiter.until_ready(50, "get_market").await;
        let result = crate::apis::systems_api::get_market(
            &self.configuration,
            system_symbol,
            waypoint_symbol,
        )
        .await?;

        Ok(result)
    }

    /// Get the shipyard for a waypoint. Requires a waypoint that has the `Shipyard` trait to use. Send a ship to the waypoint to access data on ships that are currently available for purchase and recent transactions.
    pub async fn get_shipyard(
        &self,
        system_symbol: &str,
        waypoint_symbol: &str,
    ) -> Result<models::GetShipyard200Response, Error<GetShipyardError>> {
        self.limiter.until_ready(50, "get_shipyard").await;
        let result = crate::apis::systems_api::get_shipyard(
            &self.configuration,
            system_symbol,
            waypoint_symbol,
        )
        .await?;

        Ok(result)
    }

    /// Get the details of a system.
    pub async fn get_system(
        &self,
        system_symbol: &str,
    ) -> Result<models::GetSystem200Response, Error<GetSystemError>> {
        self.limiter.until_ready(50, "get_system").await;
        let result =
            crate::apis::systems_api::get_system(&self.configuration, system_symbol).await?;

        Ok(result)
    }

    /// Return a paginated list of all of the waypoints for a given system.  If a waypoint is uncharted, it will return the `Uncharted` trait instead of its actual traits.
    pub async fn get_system_waypoints(
        &self,
        system_symbol: &str,
        page: Option<i32>,
        limit: Option<i32>,
        r#type: Option<models::WaypointType>,
        traits: Option<models::GetSystemWaypointsTraitsParameter>,
    ) -> Result<models::GetSystemWaypoints200Response, Error<GetSystemWaypointsError>> {
        self.limiter.until_ready(50, "get_system_waypoints").await;
        let result = crate::apis::systems_api::get_system_waypoints(
            &self.configuration,
            system_symbol,
            page,
            limit,
            r#type,
            traits,
        )
        .await?;

        Ok(result)
    }

    /// Returns ALL waypoints for a given system.  If a waypoint is uncharted, it will return the `Uncharted` trait instead of its actual traits.
    pub async fn get_all_waypoints(
        &self,
        system_symbol: &str,
        limit: i32,
    ) -> Result<Vec<models::Waypoint>, Error<GetSystemWaypointsError>> {
        self.get_all_waypoints_with_traits(system_symbol, None, limit)
            .await
    }

    /// Returns ALL waypoints for a given system and with the given traits.  If a waypoint is uncharted, it will return the `Uncharted` trait instead of its actual traits, but still only the waypoints that have that trait.
    pub async fn get_all_waypoints_with_traits(
        &self,
        system_symbol: &str,
        traits: Option<models::GetSystemWaypointsTraitsParameter>,
        limit: i32,
    ) -> Result<Vec<models::Waypoint>, Error<GetSystemWaypointsError>> {
        if !(1..=20).contains(&limit) {
            panic!("Invalid limit must be between 1 and 20");
        }
        let mut current_page = 1;

        let mut waypoints = Vec::new();

        loop {
            let page = self
                .get_system_waypoints(
                    system_symbol,
                    Some(current_page),
                    Some(limit),
                    None,
                    traits.clone(),
                )
                .await?;
            waypoints.extend(page.data);
            let total = page.meta.total;

            debug!(
                "limit: {}, page {} of {}, waypoints: {} of {}",
                limit,
                current_page,
                total / limit + 1,
                waypoints.len(),
                total
            );

            if waypoints.len() >= total.try_into().unwrap() {
                break;
            }

            current_page += 1;
        }
        Ok(waypoints)
    }

    /// Return a paginated list of all systems.
    pub async fn get_systems(
        &self,
        page: Option<i32>,
        limit: Option<i32>,
    ) -> Result<models::GetSystems200Response, Error<GetSystemsError>> {
        self.limiter.until_ready(50, "get_systems").await;
        let result =
            crate::apis::systems_api::get_systems(&self.configuration, page, limit).await?;

        Ok(result)
    }

    /// Return a list of ALL systems by paginating over all pages.
    pub async fn get_all_systems(&self, limit: i32) -> Result<Vec<System>, Error<GetSystemsError>> {
        if !(1..=20).contains(&limit) {
            panic!("Invalid limit must be between 1 and 20");
        }
        let mut current_page = 1;

        let mut systems = Vec::new();

        loop {
            let page = self.get_systems(Some(current_page), Some(limit)).await?;
            systems.extend(page.data);
            let total = page.meta.total;

            debug!(
                "limit: {}, page {} of {}, systems: {} of {}",
                limit,
                current_page,
                total / limit + 1,
                systems.len(),
                total
            );

            if systems.len() >= total.try_into().unwrap() {
                break;
            }

            current_page += 1;
        }

        debug!("total systems: {}", systems.len());

        Ok(systems)
    }

    /// View the details of a waypoint.  If the waypoint is uncharted, it will return the 'Uncharted' trait instead of its actual traits.
    pub async fn get_waypoint(
        &self,
        system_symbol: &str,
        waypoint_symbol: &str,
    ) -> Result<models::GetWaypoint200Response, Error<GetWaypointError>> {
        self.limiter.until_ready(50, "get_waypoint").await;
        let result = crate::apis::systems_api::get_waypoint(
            &self.configuration,
            system_symbol,
            waypoint_symbol,
        )
        .await?;

        Ok(result)
    }

    /// Supply a construction site with the specified good. Requires a waypoint with a property of `isUnderConstruction` to be true.  The good must be in your ship's cargo. The good will be removed from your ship's cargo and added to the construction site's materials.
    pub async fn supply_construction(
        &self,
        system_symbol: &str,
        waypoint_symbol: &str,
        supply_construction_request: models::SupplyConstructionRequest,
    ) -> Result<models::SupplyConstruction201Response, Error<SupplyConstructionError>> {
        if self.configuration.bearer_access_token.is_none() {
            panic!("Invalid bearer_access_token");
        }
        self.limiter.until_ready(50, "supply_construction").await;
        let result = crate::apis::systems_api::supply_construction(
            &self.configuration,
            system_symbol,
            waypoint_symbol,
            supply_construction_request,
        )
        .await?;

        Ok(result)
    }

    pub fn system_symbol(waypoint_symbol: &str) -> String {
        let waypoint_symbol_split = waypoint_symbol.split("-").collect::<Vec<&str>>();

        let system_symbol = format!("{}-{}", waypoint_symbol_split[0], waypoint_symbol_split[1]);
        system_symbol
    }
}

// #[cfg(test)]
// mod tests {

//     #[tokio::test]
//     #[ignore]
//     async fn get_systems_test() {
//         let api = super::Api::new(None, 550, std::num::NonZeroU32::new(2).unwrap());
//         let response_json = api.get_all_systems_json().await.unwrap();
//         let result = api.get_all_systems(20).await.unwrap();
//         assert_eq!(response_json, result, "response_json != result!()");
//     }

//     #[tokio::test]
//     #[ignore]
//     async fn get_systems_json_test() {
//         let api = super::Api::new(None, 550, std::num::NonZeroU32::new(2).unwrap());
//         let response_json = api.get_all_systems_json().await.unwrap();
//         assert!(!response_json.is_empty());
//     }

//     #[tokio::test]
//     #[should_panic]
//     async fn get_system_waypoints_test() {
//         let api = super::Api::new(None, 550, std::num::NonZeroU32::new(2).unwrap());
//         let result = api.get_all_waypoints("test", 22).await;
//         assert!(result.is_ok());
//     }

//     #[tokio::test]
//     #[should_panic]
//     async fn get_my_agent_test() {
//         let api = super::Api::new(None, 550, std::num::NonZeroU32::new(2).unwrap());
//         let response = api.get_my_agent().await;
//         assert!(response.is_ok());
//     }
// }
