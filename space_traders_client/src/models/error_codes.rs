/// General Error Codes
pub const UNAUTHORIZED: u32 = 401;
pub const COOLDOWN_CONFLICT_ERROR: u32 = 4000;
pub const WAYPOINT_NO_ACCESS_ERROR: u32 = 4001;

/// Account Error Codes
pub const TOKEN_EMPTY_ERROR: u32 = 4100;
pub const TOKEN_MISSING_SUBJECT_ERROR: u32 = 4101;
pub const TOKEN_INVALID_SUBJECT_ERROR: u32 = 4102;
pub const MISSING_TOKEN_REQUEST_ERROR: u32 = 4103;
pub const INVALID_TOKEN_REQUEST_ERROR: u32 = 4104;
pub const INVALID_TOKEN_SUBJECT_ERROR: u32 = 4105;
pub const ACCOUNT_NOT_EXISTS_ERROR: u32 = 4106;
pub const AGENT_NOT_EXISTS_ERROR: u32 = 4107;
pub const ACCOUNT_HAS_NO_AGENT_ERROR: u32 = 4108;
pub const REGISTER_AGENT_EXISTS_ERROR: u32 = 4109;
pub const REGISTER_AGENT_SYMBOL_RESERVED_ERROR: u32 = 4110;
pub const REGISTER_AGENT_CONFLICT_SYMBOL_ERROR: u32 = 4111;

/// Ship Error Codes
pub const NAVIGATE_IN_TRANSIT_ERROR: u32 = 4200;
pub const NAVIGATE_INVALID_DESTINATION_ERROR: u32 = 4201;
pub const NAVIGATE_OUTSIDE_SYSTEM_ERROR: u32 = 4202;
pub const NAVIGATE_INSUFFICIENT_FUEL_ERROR: u32 = 4203;
pub const NAVIGATE_SAME_DESTINATION_ERROR: u32 = 4204;
pub const SHIP_EXTRACT_INVALID_WAYPOINT_ERROR: u32 = 4205;
pub const SHIP_EXTRACT_PERMISSION_ERROR: u32 = 4206;
pub const SHIP_JUMP_NO_SYSTEM_ERROR: u32 = 4207;
pub const SHIP_JUMP_SAME_SYSTEM_ERROR: u32 = 4208;
pub const SHIP_JUMP_MISSING_MODULE_ERROR: u32 = 4210;
pub const SHIP_JUMP_NO_VALID_WAYPOINT_ERROR: u32 = 4211;
pub const SHIP_JUMP_MISSING_ANTIMATTER_ERROR: u32 = 4212;
pub const SHIP_IN_TRANSIT_ERROR: u32 = 4214;
pub const SHIP_MISSING_SENSOR_ARRAYS_ERROR: u32 = 4215;
pub const PURCHASE_SHIP_CREDITS_ERROR: u32 = 4216;
pub const SHIP_CARGO_EXCEEDS_LIMIT_ERROR: u32 = 4217;
pub const SHIP_CARGO_MISSING_ERROR: u32 = 4218;
pub const SHIP_CARGO_UNIT_COUNT_ERROR: u32 = 4219;
pub const SHIP_SURVEY_VERIFICATION_ERROR: u32 = 4220;
pub const SHIP_SURVEY_EXPIRATION_ERROR: u32 = 4221;
pub const SHIP_SURVEY_WAYPOINT_TYPE_ERROR: u32 = 4222;
pub const SHIP_SURVEY_ORBIT_ERROR: u32 = 4223;
pub const SHIP_SURVEY_EXHAUSTED_ERROR: u32 = 4224;
pub const SHIP_REFUEL_DOCKED_ERROR: u32 = 4225;
pub const SHIP_REFUEL_INVALID_WAYPOINT_ERROR: u32 = 4226;
pub const SHIP_MISSING_MOUNTS_ERROR: u32 = 4227;
pub const SHIP_CARGO_FULL_ERROR: u32 = 4228;
pub const SHIP_JUMP_FROM_GATE_TO_GATE_ERROR: u32 = 4229;
pub const WAYPOINT_CHARTED_ERROR: u32 = 4230;
pub const SHIP_TRANSFER_SHIP_NOT_FOUND: u32 = 4231;
pub const SHIP_TRANSFER_AGENT_CONFLICT: u32 = 4232;
pub const SHIP_TRANSFER_SAME_SHIP_CONFLICT: u32 = 4233;
pub const SHIP_TRANSFER_LOCATION_CONFLICT: u32 = 4234;
pub const WARP_INSIDE_SYSTEM_ERROR: u32 = 4235;
pub const SHIP_NOT_IN_ORBIT_ERROR: u32 = 4236;
pub const SHIP_INVALID_REFINERY_GOOD_ERROR: u32 = 4237;
pub const SHIP_INVALID_REFINERY_TYPE_ERROR: u32 = 4238;
pub const SHIP_MISSING_REFINERY_ERROR: u32 = 4239;
pub const SHIP_MISSING_SURVEYOR_ERROR: u32 = 4240;
pub const SHIP_MISSING_WARP_DRIVE_ERROR: u32 = 4241;
pub const SHIP_MISSING_MINERAL_PROCESSOR_ERROR: u32 = 4242;
pub const SHIP_MISSING_MINING_LASERS_ERROR: u32 = 4243;
pub const SHIP_NOT_DOCKED_ERROR: u32 = 4244;
pub const PURCHASE_SHIP_NOT_PRESENT_ERROR: u32 = 4245;
pub const SHIP_MOUNT_NO_SHIPYARD_ERROR: u32 = 4246;
pub const SHIP_MISSING_MOUNT_ERROR: u32 = 4247;
pub const SHIP_MOUNT_INSUFFICIENT_CREDITS_ERROR: u32 = 4248;
pub const SHIP_MISSING_POWER_ERROR: u32 = 4249;
pub const SHIP_MISSING_SLOTS_ERROR: u32 = 4250;
pub const SHIP_MISSING_CREW_ERROR: u32 = 4252;
pub const SHIP_EXTRACT_DESTABILIZED_ERROR: u32 = 4253;
pub const SHIP_MODULE_NO_SHIPYARD: u32 = 4266;
pub const SHIP_MODULE_NOT_INSTALLED: u32 = 4267;
pub const SHIP_MODULE_INSUFFICIENT_CREDITS: u32 = 4268;

/// Contract Error Codes
pub const ACCEPT_CONTRACT_NOT_AUTHORIZED_ERROR: u32 = 4500;
pub const ACCEPT_CONTRACT_CONFLICT_ERROR: u32 = 4501;
pub const FULFILL_CONTRACT_DELIVERY_ERROR: u32 = 4502;
pub const CONTRACT_DEADLINE_ERROR: u32 = 4503;
pub const CONTRACT_FULFILLED_ERROR: u32 = 4504;
pub const CONTRACT_NOT_ACCEPTED_ERROR: u32 = 4505;
pub const CONTRACT_NOT_AUTHORIZED_ERROR: u32 = 4506;
pub const SHIP_DELIVER_TERMS_ERROR: u32 = 4508;
pub const SHIP_DELIVER_FULFILLED_ERROR: u32 = 4509;
pub const SHIP_DELIVER_INVALID_LOCATION_ERROR: u32 = 4510;
pub const EXISTING_CONTRACT_ERROR: u32 = 4511;

/// Market Error Codes
pub const MARKET_TRADE_INSUFFICIENT_CREDITS_ERROR: u32 = 4600;
pub const MARKET_TRADE_NO_PURCHASE_ERROR: u32 = 4601;
pub const MARKET_TRADE_NOT_SOLD_ERROR: u32 = 4602;
pub const MARKET_NOT_FOUND_ERROR: u32 = 4603;
pub const MARKET_TRADE_UNIT_LIMIT_ERROR: u32 = 4604;

/// Faction Error Codes
pub const WAYPOINT_NO_FACTION_ERROR: u32 = 4700;

/// Construction Error Codes
pub const CONSTRUCTION_MATERIAL_NOT_REQUIRED: u32 = 4800;
pub const CONSTRUCTION_MATERIAL_FULFILLED: u32 = 4801;
pub const SHIP_CONSTRUCTION_INVALID_LOCATION_ERROR: u32 = 4802;
