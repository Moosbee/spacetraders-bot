use async_recursion::async_recursion;
use chrono::{DateTime, Local};

use log::{error, info};

use serde_json::Value;
use space_traders_client::apis::agents_api::get_my_agent;
use space_traders_client::apis::configuration::Configuration;
use space_traders_client::apis::contracts_api::{
    accept_contract, deliver_contract, fulfill_contract, get_contract, get_contracts,
};
use space_traders_client::apis::default_api::{register, RegisterError};
use space_traders_client::apis::fleet_api::{
    dock_ship, extract_resources, get_my_ship, get_my_ships, navigate_ship, orbit_ship,
    purchase_ship, refuel_ship, sell_cargo,
};
use space_traders_client::apis::systems_api::{get_shipyard, get_system_waypoints, get_waypoint};
use space_traders_client::apis::Error;
use space_traders_client::models::{self, Faction, FactionSymbol};
use space_traders_client::models::{
    Agent, Contract, DeliverContractRequest, NavigateShipRequest, PurchaseShipRequest,
    Register201ResponseData, RegisterRequest, SellCargoRequest, Ship, ShipCargo, ShipFuel, ShipNav,
    ShipType, Shipyard, Waypoint,
};

// General error codes
const UNAUTHORIZED: u32 = 401;
const COOLDOWN_CONFLICT_ERROR: u32 = 4000;
const WAYPOINT_NO_ACCESS_ERROR: u32 = 4001;
const TOKEN_EMPTY_ERROR: u32 = 4100;
// Account error codes
const TOKEN_MISSING_SUBJECT_ERROR: u32 = 4101;
const TOKEN_INVALID_SUBJECT_ERROR: u32 = 4102;
const MISSING_TOKEN_REQUEST_ERROR: u32 = 4103;
const INVALID_TOKEN_REQUEST_ERROR: u32 = 4104;
const INVALID_TOKEN_SUBJECT_ERROR: u32 = 4105;
const ACCOUNT_NOT_EXISTS_ERROR: u32 = 4106;
const AGENT_NOT_EXISTS_ERROR: u32 = 4107;
const ACCOUNT_HAS_NO_AGENT_ERROR: u32 = 4108;
const REGISTER_AGENT_EXISTS_ERROR: u32 = 4109;
// Ship error codes
const NAVIGATE_IN_TRANSIT_ERROR: u32 = 4200;
const NAVIGATE_INVALID_DESTINATION_ERROR: u32 = 4201;
const NAVIGATE_OUTSIDE_SYSTEM_ERROR: u32 = 4202;
const NAVIGATE_INSUFFICIENT_FUEL_ERROR: u32 = 4203;
const NAVIGATE_SAME_DESTINATION_ERROR: u32 = 4204;
const SHIP_EXTRACT_INVALID_WAYPOINT_ERROR: u32 = 4205;
const SHIP_EXTRACT_PERMISSION_ERROR: u32 = 4206;
const SHIP_JUMP_NO_SYSTEM_ERROR: u32 = 4207;
const SHIP_JUMP_SAME_SYSTEM_ERROR: u32 = 4208;
const SHIP_JUMP_MISSING_MODULE_ERROR: u32 = 4210;
const SHIP_JUMP_NO_VALID_WAYPOINT_ERROR: u32 = 4211;
const SHIP_JUMP_MISSING_ANTIMATTER_ERROR: u32 = 4212;
const SHIP_IN_TRANSIT_ERROR: u32 = 4214;
const SHIP_MISSING_SENSOR_ARRAYS_ERROR: u32 = 4215;
const PURCHASE_SHIP_CREDITS_ERROR: u32 = 4216;
const SHIP_CARGO_EXCEEDS_LIMIT_ERROR: u32 = 4217;
const SHIP_CARGO_MISSING_ERROR: u32 = 4218;
const SHIP_CARGO_UNIT_COUNT_ERROR: u32 = 4219;
const SHIP_SURVEY_VERIFICATION_ERROR: u32 = 4220;
const SHIP_SURVEY_EXPIRATION_ERROR: u32 = 4221;
const SHIP_SURVEY_WAYPOINT_TYPE_ERROR: u32 = 4222;
const SHIP_SURVEY_ORBIT_ERROR: u32 = 4223;
const SHIP_SURVEY_EXHAUSTED_ERROR: u32 = 4224;
const SHIP_REFUEL_DOCKED_ERROR: u32 = 4225;
const SHIP_REFUEL_INVALID_WAYPOINT_ERROR: u32 = 4226;
const SHIP_MISSING_MOUNTS_ERROR: u32 = 4227;
const SHIP_CARGO_FULL_ERROR: u32 = 4228;
const SHIP_JUMP_FROM_GATE_TO_GATE_ERROR: u32 = 4229;
const WAYPOINT_CHARTED_ERROR: u32 = 4230;
const SHIP_TRANSFER_SHIP_NOT_FOUND: u32 = 4231;
const SHIP_TRANSFER_AGENT_CONFLICT: u32 = 4232;
const SHIP_TRANSFER_SAME_SHIP_CONFLICT: u32 = 4233;
const SHIP_TRANSFER_LOCATION_CONFLICT: u32 = 4234;
const WARP_INSIDE_SYSTEM_ERROR: u32 = 4235;
const SHIP_NOT_IN_ORBIT_ERROR: u32 = 4236;
const SHIP_INVALID_REFINERY_GOOD_ERROR: u32 = 4237;
const SHIP_INVALID_REFINERY_TYPE_ERROR: u32 = 4238;
const SHIP_MISSING_REFINERY_ERROR: u32 = 4239;
const SHIP_MISSING_SURVEYOR_ERROR: u32 = 4240;
// Contract error codes
const ACCEPT_CONTRACT_NOT_AUTHORIZED_ERROR: u32 = 4500;
const ACCEPT_CONTRACT_CONFLICT_ERROR: u32 = 4501;
const FULFILL_CONTRACT_DELIVERY_ERROR: u32 = 4502;
const CONTRACT_DEADLINE_ERROR: u32 = 4503;
const CONTRACT_FULFILLED_ERROR: u32 = 4504;
const CONTRACT_NOT_ACCEPTED_ERROR: u32 = 4505;
const CONTRACT_NOT_AUTHORIZED_ERROR: u32 = 4506;
const SHIP_DELIVER_TERMS_ERROR: u32 = 4508;
const SHIP_DELIVER_FULFILLED_ERROR: u32 = 4509;
const SHIP_DELIVER_INVALID_LOCATION_ERROR: u32 = 4510;
// Market error codes
const MARKET_TRADE_INSUFFICIENT_CREDITS_ERROR: u32 = 4600;
const MARKET_TRADE_NO_PURCHASE_ERROR: u32 = 4601;
const MARKET_TRADE_NOT_SOLD_ERROR: u32 = 4602;
const MARKET_NOT_FOUND_ERROR: u32 = 4603;
const MARKET_TRADE_UNIT_LIMIT_ERROR: u32 = 4604;

#[derive(Debug)]
struct ErrorInfo<T> {
    code: i32,
    message: String,
    data: Option<T>,
}

#[derive(Debug)]
pub struct Api {
    configuration: Configuration,
}

impl Api {
    pub fn new(access_token: String) -> Api {
        Api {
            configuration: Configuration {
                bearer_access_token: Some(access_token),
                ..Default::default()
            },
        }
    }
}
