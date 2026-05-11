/* eslint-disable */
import { TypedDocumentNode as DocumentNode } from '@graphql-typed-document-node/core';
export type Maybe<T> = T | null;
export type InputMaybe<T> = T | null | undefined;
export type Exact<T extends { [key: string]: unknown }> = { [K in keyof T]: T[K] };
export type MakeOptional<T, K extends keyof T> = Omit<T, K> & { [SubKey in K]?: Maybe<T[SubKey]> };
export type MakeMaybe<T, K extends keyof T> = Omit<T, K> & { [SubKey in K]: Maybe<T[SubKey]> };
export type MakeEmpty<T extends { [key: string]: unknown }, K extends keyof T> = { [_ in K]?: never };
export type Incremental<T> = T | { [P in keyof T]?: P extends ' $fragmentName' | '__typename' ? T[P] : never };
/** All built-in and custom scalars, mapped to their actual values */
export type Scalars = {
  ID: { input: string; output: string; }
  String: { input: string; output: string; }
  Boolean: { input: boolean; output: boolean; }
  Int: { input: number; output: number; }
  Float: { input: number; output: number; }
  /**
   * Implement the DateTime<Utc> scalar
   *
   * The input/output is a string in RFC3339 format.
   */
  DateTime: { input: string; output: string; }
  /** A scalar that can represent any JSON Object value. */
  JSONObject: { input: any; output: any; }
};

/**
 * ActivityLevel : The activity level of a trade good. If the good is an import, this represents how strong consumption is. If the good is an export, this represents how strong the production is for the good. When activity is strong, consumption or production is near maximum capacity. When activity is weak, consumption or production is near minimum capacity.
 * The activity level of a trade good. If the good is an import, this represents how strong consumption is. If the good is an export, this represents how strong the production is for the good. When activity is strong, consumption or production is near maximum capacity. When activity is weak, consumption or production is near minimum capacity.
 */
export enum ActivityLevel {
  Growing = 'GROWING',
  Restricted = 'RESTRICTED',
  Strong = 'STRONG',
  Weak = 'WEAK'
}

export type Agent = {
  __typename?: 'Agent';
  accountId?: Maybe<Scalars['String']['output']>;
  createdAt: Scalars['DateTime']['output'];
  credits: Scalars['Int']['output'];
  headquarters: Scalars['String']['output'];
  headquartersSystem?: Maybe<System>;
  headquartersWaypoint?: Maybe<Waypoint>;
  history: Array<Agent>;
  id: Scalars['Int']['output'];
  shipCount: Scalars['Int']['output'];
  startingFaction: Scalars['String']['output'];
  symbol: Scalars['String']['output'];
};

export enum AssignLevel {
  /** Ship is assigned to a waypoint and is there */
  Active = 'ACTIVE',
  /** Ship is at the waypoint but not active */
  Inactive = 'INACTIVE',
  /** Ship is assigned to a waypoint and is on its way */
  OnTheWay = 'ON_THE_WAY'
}

export type AssignedShip = {
  __typename?: 'AssignedShip';
  level: AssignLevel;
  shipSymbol: Scalars['String']['output'];
};

export type AssignmentStatus = ChartingStatus | ConstructionStatus | ContractStatus | ManuelStatus | MiningStatus | ScraperStatus | TraderStatus | TransferStatus;

export type AutopilotRoute = {
  __typename?: 'AutopilotRoute';
  connections: Array<ConcreteConnection>;
  totalApiRequests: Scalars['Int']['output'];
  totalDistance: Scalars['Float']['output'];
  totalFuelCost: Scalars['Float']['output'];
  totalTravelTime: Scalars['Float']['output'];
};

export type AutopilotState = {
  __typename?: 'AutopilotState';
  arrival: Scalars['DateTime']['output'];
  departureTime: Scalars['DateTime']['output'];
  destinationSymbol: Scalars['String']['output'];
  destinationSystem?: Maybe<System>;
  destinationSystemSymbol: Scalars['String']['output'];
  destinationWaypoint?: Maybe<Waypoint>;
  distance: Scalars['Float']['output'];
  fuelCost: Scalars['Int']['output'];
  originSymbol: Scalars['String']['output'];
  originSystem?: Maybe<System>;
  originSystemSymbol: Scalars['String']['output'];
  originWaypoint?: Maybe<Waypoint>;
  route: AutopilotRoute;
  travelTime: Scalars['Float']['output'];
};

export type BudgetInfo = {
  __typename?: 'BudgetInfo';
  currentFunds: Scalars['Int']['output'];
  ironReserve: Scalars['Int']['output'];
  reservations: Array<ReservedFund>;
  reservedAmount: Scalars['Int']['output'];
  spendable: Scalars['Int']['output'];
};

export type CargoState = {
  __typename?: 'CargoState';
  capacity: Scalars['Int']['output'];
  inventory: Array<CargoVolume>;
  units: Scalars['Int']['output'];
};

export type CargoVolume = {
  __typename?: 'CargoVolume';
  symbol: TradeSymbol;
  units: Scalars['Int']['output'];
};

export type ChannelInfo = {
  __typename?: 'ChannelInfo';
  freeCapacity: Scalars['Int']['output'];
  state: ChannelState;
  totalCapacity: Scalars['Int']['output'];
  usedCapacity: Scalars['Int']['output'];
};

export enum ChannelState {
  Closed = 'CLOSED',
  Open = 'OPEN'
}

export type ChartManagerInfo = {
  __typename?: 'ChartManagerInfo';
  busy: Scalars['Boolean']['output'];
  channelState: ChannelInfo;
};

export type ChartTransaction = {
  __typename?: 'ChartTransaction';
  id: Scalars['Int']['output'];
  ship?: Maybe<Ship>;
  /** The symbol of the ship. */
  shipSymbol: Scalars['String']['output'];
  /** The timestamp of the transaction. */
  timestamp: Scalars['DateTime']['output'];
  /** The total price of the transaction. */
  totalPrice: Scalars['Int']['output'];
  waypoint?: Maybe<Waypoint>;
  /** The symbol of the waypoint. */
  waypointSymbol: Scalars['String']['output'];
};

export type ChartTransactionPage = {
  __typename?: 'ChartTransactionPage';
  items: Array<ChartTransaction>;
  page: Scalars['Int']['output'];
  pageSize?: Maybe<Scalars['Int']['output']>;
  totalCount: Scalars['Int']['output'];
};

export type ChartingConfig = {
  __typename?: 'ChartingConfig';
  chartingProbeCount: Scalars['Int']['output'];
};

export type ChartingStatus = {
  __typename?: 'ChartingStatus';
  cycle?: Maybe<Scalars['Int']['output']>;
  waitingForManager: Scalars['Boolean']['output'];
  waypoint?: Maybe<Waypoint>;
  waypointSymbol?: Maybe<Scalars['String']['output']>;
};

export type ConcreteConnection = JumpConnection | NavigateConnection | WarpConnection;

export type Condition = {
  __typename?: 'Condition';
  condition: Scalars['Float']['output'];
  integrity: Scalars['Float']['output'];
};

export type ConditionState = {
  __typename?: 'ConditionState';
  engine: Condition;
  frame: Condition;
  reactor: Condition;
};

export type Config = {
  __typename?: 'Config';
  antimatterPrice: Scalars['Int']['output'];
  controlActive: Scalars['Boolean']['output'];
  controlStartSleep: Scalars['Int']['output'];
  defaultProfit: Scalars['Int']['output'];
  defaultPurchasePrice: Scalars['Int']['output'];
  defaultSellPrice: Scalars['Int']['output'];
  expand: Scalars['Boolean']['output'];
  extraMiningTransporter: Scalars['Int']['output'];
  fuelCost: Scalars['Int']['output'];
  ignoreEngineeredAsteroids: Scalars['Boolean']['output'];
  ironReserve: Scalars['Int']['output'];
  marginPercentage: Scalars['Float']['output'];
  marketBlacklist: Array<TradeSymbol>;
  marketsPerShip: Scalars['Int']['output'];
  markupPercentage: Scalars['Float']['output'];
  maxMinersPerWaypoint: Scalars['Int']['output'];
  maxUpdateInterval: Scalars['Int']['output'];
  miningEjectList: Array<TradeSymbol>;
  miningPreferList: Array<TradeSymbol>;
  miningShipsPerWaypoint: Scalars['Int']['output'];
  miningWaypointsPerSystem: Scalars['Int']['output'];
  purchaseMultiplier: Scalars['Float']['output'];
  scrapAgents: Scalars['Boolean']['output'];
  scrapperStartSleep: Scalars['Int']['output'];
  shipPurchaseAmount: Scalars['Int']['output'];
  shipPurchasePercentile: Scalars['Float']['output'];
  shipPurchaseStop: Scalars['Boolean']['output'];
  socketAddress: Scalars['String']['output'];
  stopAllUnstable: Scalars['Boolean']['output'];
  tradeMode: TradeMode;
  tradeProfitThreshold: Scalars['Int']['output'];
  transportCapacityPerWaypoint: Scalars['Int']['output'];
  unstableSinceTimeout: Scalars['Int']['output'];
  updateAllSystems: Scalars['Boolean']['output'];
};

export type ConstructionConfig = {
  __typename?: 'ConstructionConfig';
  constructionShipCount: Scalars['Int']['output'];
  constructionWaypoint: Scalars['String']['output'];
};

export type ConstructionManagerInfo = {
  __typename?: 'ConstructionManagerInfo';
  busy: Scalars['Boolean']['output'];
  channelState: ChannelInfo;
  runningShipments: Array<ConstructionShipment>;
};

export type ConstructionMaterial = {
  __typename?: 'ConstructionMaterial';
  createdAt: Scalars['DateTime']['output'];
  fulfilled: Scalars['Int']['output'];
  id: Scalars['Int']['output'];
  marketTransactionSummary: TransactionSummary;
  required: Scalars['Int']['output'];
  shipments: ConstructionShipmentPage;
  tradeSymbol: TradeSymbol;
  tradeSymbolInfo: TradeSymbolInfo;
  updatedAt: Scalars['DateTime']['output'];
  waypoint?: Maybe<Waypoint>;
  waypointSymbol: Scalars['String']['output'];
};


export type ConstructionMaterialShipmentsArgs = {
  page?: InputMaybe<Scalars['Int']['input']>;
  pageSize?: InputMaybe<Scalars['Int']['input']>;
};

export type ConstructionMaterialBy =
  { system: Scalars['String']['input']; tradeSymbol?: never; waypoint?: never; }
  |  { system?: never; tradeSymbol: TradeSymbol; waypoint?: never; }
  |  { system?: never; tradeSymbol?: never; waypoint: Scalars['String']['input']; };

export type ConstructionMaterialPage = {
  __typename?: 'ConstructionMaterialPage';
  items: Array<ConstructionMaterial>;
  page: Scalars['Int']['output'];
  pageSize?: Maybe<Scalars['Int']['output']>;
  totalCount: Scalars['Int']['output'];
};

export type ConstructionShipment = {
  __typename?: 'ConstructionShipment';
  constructionSiteWaypoint: Scalars['String']['output'];
  constructionWaypoint?: Maybe<Waypoint>;
  createdAt: Scalars['DateTime']['output'];
  id: Scalars['Int']['output'];
  marketTransactionSummary: TransactionSummary;
  marketTransactions: MarketTransactionPage;
  material?: Maybe<ConstructionMaterial>;
  materialId: Scalars['Int']['output'];
  purchaseMarketTradeGood?: Maybe<MarketTradeGood>;
  purchaseSiteWaypoint: Scalars['String']['output'];
  purchaseWaypoint?: Maybe<Waypoint>;
  reservation?: Maybe<ReservedFund>;
  reservedFund?: Maybe<Scalars['Int']['output']>;
  ship?: Maybe<Ship>;
  shipSymbol: Scalars['String']['output'];
  status: ShipmentStatus;
  tradeSymbol: TradeSymbol;
  tradeSymbolInfo: TradeSymbolInfo;
  units: Scalars['Int']['output'];
  updatedAt: Scalars['DateTime']['output'];
};


export type ConstructionShipmentMarketTransactionsArgs = {
  page?: InputMaybe<Scalars['Int']['input']>;
  pageSize?: InputMaybe<Scalars['Int']['input']>;
};

export type ConstructionShipmentBy =
  { material: Scalars['Int']['input']; shipSymbol?: never; system?: never; tradeSymbol?: never; waypoint?: never; }
  |  { material?: never; shipSymbol: Scalars['String']['input']; system?: never; tradeSymbol?: never; waypoint?: never; }
  |  { material?: never; shipSymbol?: never; system: Scalars['String']['input']; tradeSymbol?: never; waypoint?: never; }
  |  { material?: never; shipSymbol?: never; system?: never; tradeSymbol: TradeSymbol; waypoint?: never; }
  |  { material?: never; shipSymbol?: never; system?: never; tradeSymbol?: never; waypoint: Scalars['String']['input']; };

export type ConstructionShipmentPage = {
  __typename?: 'ConstructionShipmentPage';
  items: Array<ConstructionShipment>;
  page: Scalars['Int']['output'];
  pageSize?: Maybe<Scalars['Int']['output']>;
  totalCount: Scalars['Int']['output'];
};

export type ConstructionStatus = {
  __typename?: 'ConstructionStatus';
  cycle?: Maybe<Scalars['Int']['output']>;
  shipment?: Maybe<ConstructionShipment>;
  shipmentId?: Maybe<Scalars['Int']['output']>;
  shippingStatus?: Maybe<ShippingStatus>;
  waitingForManager: Scalars['Boolean']['output'];
};

export type Contract = {
  __typename?: 'Contract';
  accepted: Scalars['Boolean']['output'];
  contractType: ContractType;
  createdAt: Scalars['DateTime']['output'];
  deadline: Scalars['String']['output'];
  deadlineToAccept?: Maybe<Scalars['String']['output']>;
  deliveries: ContractDeliveryPage;
  factionSymbol: Scalars['String']['output'];
  fulfilled: Scalars['Boolean']['output'];
  id: Scalars['String']['output'];
  marketTransactionSummary: TransactionSummary;
  marketTransactions: MarketTransactionPage;
  onAccepted: Scalars['Int']['output'];
  onFulfilled: Scalars['Int']['output'];
  reservation?: Maybe<ReservedFund>;
  reservedFund?: Maybe<Scalars['Int']['output']>;
  shipments: ContractShipmentPage;
  updatedAt: Scalars['DateTime']['output'];
};


export type ContractDeliveriesArgs = {
  page?: InputMaybe<Scalars['Int']['input']>;
  pageSize?: InputMaybe<Scalars['Int']['input']>;
};


export type ContractMarketTransactionsArgs = {
  page?: InputMaybe<Scalars['Int']['input']>;
  pageSize?: InputMaybe<Scalars['Int']['input']>;
};


export type ContractShipmentsArgs = {
  page?: InputMaybe<Scalars['Int']['input']>;
  pageSize?: InputMaybe<Scalars['Int']['input']>;
};

export type ContractBy =
  { faction: FactionSymbol; };

export type ContractConfig = {
  __typename?: 'ContractConfig';
  contractShipCount: Scalars['Int']['output'];
};

export type ContractDelivery = {
  __typename?: 'ContractDelivery';
  contract?: Maybe<Contract>;
  contractId: Scalars['String']['output'];
  contractShipment: ContractShipmentPage;
  destinationSymbol: Scalars['String']['output'];
  tradeSymbol: TradeSymbol;
  tradeSymbolInfo: TradeSymbolInfo;
  unitsFulfilled: Scalars['Int']['output'];
  unitsRequired: Scalars['Int']['output'];
  waypoint?: Maybe<Waypoint>;
};


export type ContractDeliveryContractShipmentArgs = {
  page?: InputMaybe<Scalars['Int']['input']>;
  pageSize?: InputMaybe<Scalars['Int']['input']>;
};

export type ContractDeliveryBy =
  { contract: Scalars['String']['input']; tradeSymbol?: never; waypoint?: never; }
  |  { contract?: never; tradeSymbol: TradeSymbol; waypoint?: never; }
  |  { contract?: never; tradeSymbol?: never; waypoint: Scalars['String']['input']; };

export type ContractDeliveryPage = {
  __typename?: 'ContractDeliveryPage';
  items: Array<ContractDelivery>;
  page: Scalars['Int']['output'];
  pageSize?: Maybe<Scalars['Int']['output']>;
  totalCount: Scalars['Int']['output'];
};

export type ContractManagerInfo = {
  __typename?: 'ContractManagerInfo';
  busy: Scalars['Boolean']['output'];
  channelState: ChannelInfo;
  runningShipments: Array<ContractShipment>;
};

export type ContractPage = {
  __typename?: 'ContractPage';
  items: Array<Contract>;
  page: Scalars['Int']['output'];
  pageSize?: Maybe<Scalars['Int']['output']>;
  totalCount: Scalars['Int']['output'];
};

export type ContractShipment = {
  __typename?: 'ContractShipment';
  contract?: Maybe<Contract>;
  contractId: Scalars['String']['output'];
  createdAt: Scalars['DateTime']['output'];
  destinationSymbol: Scalars['String']['output'];
  destinationWaypoint?: Maybe<Waypoint>;
  id: Scalars['Int']['output'];
  purchaseMarketTradeGood?: Maybe<MarketTradeGood>;
  purchaseSymbol: Scalars['String']['output'];
  purchaseWaypoint?: Maybe<Waypoint>;
  ship?: Maybe<Ship>;
  shipSymbol: Scalars['String']['output'];
  status: ShipmentStatus;
  tradeSymbol: TradeSymbol;
  tradeSymbolInfo: TradeSymbolInfo;
  units: Scalars['Int']['output'];
  updatedAt: Scalars['DateTime']['output'];
};

export type ContractShipmentBy =
  { contract: Scalars['String']['input']; destinationWaypoint?: never; shipSymbol?: never; sourceWaypoint?: never; tradeSymbol?: never; }
  |  { contract?: never; destinationWaypoint: Scalars['String']['input']; shipSymbol?: never; sourceWaypoint?: never; tradeSymbol?: never; }
  |  { contract?: never; destinationWaypoint?: never; shipSymbol: Scalars['String']['input']; sourceWaypoint?: never; tradeSymbol?: never; }
  |  { contract?: never; destinationWaypoint?: never; shipSymbol?: never; sourceWaypoint: Scalars['String']['input']; tradeSymbol?: never; }
  |  { contract?: never; destinationWaypoint?: never; shipSymbol?: never; sourceWaypoint?: never; tradeSymbol: TradeSymbol; };

export type ContractShipmentPage = {
  __typename?: 'ContractShipmentPage';
  items: Array<ContractShipment>;
  page: Scalars['Int']['output'];
  pageSize?: Maybe<Scalars['Int']['output']>;
  totalCount: Scalars['Int']['output'];
};

export type ContractStatus = {
  __typename?: 'ContractStatus';
  contract?: Maybe<Contract>;
  contractId?: Maybe<Scalars['String']['output']>;
  contractRun?: Maybe<ContractShipment>;
  cycle?: Maybe<Scalars['Int']['output']>;
  runId?: Maybe<Scalars['Int']['output']>;
  shippingStatus?: Maybe<ShippingStatus>;
  waitingForManager: Scalars['Boolean']['output'];
};

/** Type of contract. */
export enum ContractType {
  Procurement = 'PROCUREMENT',
  Shuttle = 'SHUTTLE',
  Transport = 'TRANSPORT'
}

export type DbFleet = {
  __typename?: 'DBFleet';
  active: Scalars['Boolean']['output'];
  createdAt: Scalars['DateTime']['output'];
  fleetType: FleetType;
  id: Scalars['Int']['output'];
  systemSymbol: Scalars['String']['output'];
  updatedAt: Scalars['DateTime']['output'];
};

export type EngineInfo = {
  __typename?: 'EngineInfo';
  crewRequired?: Maybe<Scalars['Int']['output']>;
  description: Scalars['String']['output'];
  name: Scalars['String']['output'];
  powerRequired?: Maybe<Scalars['Int']['output']>;
  slotsRequired?: Maybe<Scalars['Int']['output']>;
  speed: Scalars['Int']['output'];
  symbol: ShipEngineSymbol;
  tradeSymbolInfo: TradeSymbolInfo;
};

export type Extraction = {
  __typename?: 'Extraction';
  createdAt: Scalars['DateTime']['output'];
  id: Scalars['Int']['output'];
  ship?: Maybe<Ship>;
  shipInfoAfter: Scalars['Int']['output'];
  shipInfoBefore: Scalars['Int']['output'];
  shipSymbol: Scalars['String']['output'];
  siphon: Scalars['Boolean']['output'];
  survey?: Maybe<Survey>;
  survey_signature?: Maybe<Scalars['String']['output']>;
  tradeSymbolInfo: TradeSymbolInfo;
  waypoint?: Maybe<Waypoint>;
  waypointSymbol: Scalars['String']['output'];
  yieldSymbol: TradeSymbol;
  yieldUnits: Scalars['Int']['output'];
};

export type ExtractionBy =
  { shipSymbol: Scalars['String']['input']; siphon?: never; survey?: never; system?: never; tradeSymbol?: never; waypoint?: never; }
  |  { shipSymbol?: never; siphon: Scalars['Boolean']['input']; survey?: never; system?: never; tradeSymbol?: never; waypoint?: never; }
  |  { shipSymbol?: never; siphon?: never; survey: Scalars['String']['input']; system?: never; tradeSymbol?: never; waypoint?: never; }
  |  { shipSymbol?: never; siphon?: never; survey?: never; system: Scalars['String']['input']; tradeSymbol?: never; waypoint?: never; }
  |  { shipSymbol?: never; siphon?: never; survey?: never; system?: never; tradeSymbol: TradeSymbol; waypoint?: never; }
  |  { shipSymbol?: never; siphon?: never; survey?: never; system?: never; tradeSymbol?: never; waypoint: Scalars['String']['input']; };

export type ExtractionPage = {
  __typename?: 'ExtractionPage';
  items: Array<Extraction>;
  page: Scalars['Int']['output'];
  pageSize?: Maybe<Scalars['Int']['output']>;
  totalCount: Scalars['Int']['output'];
};

export type ExtractorAssignment = {
  __typename?: 'ExtractorAssignment';
  extractions?: Maybe<Scalars['Int']['output']>;
  state: ExtractorState;
  waypointSymbol?: Maybe<Scalars['String']['output']>;
};

export enum ExtractorState {
  InvFull = 'INV_FULL',
  InTransit = 'IN_TRANSIT',
  Mining = 'MINING',
  OnCooldown = 'ON_COOLDOWN',
  Unknown = 'UNKNOWN'
}

/**
 * FactionSymbol : The symbol of the faction.
 * The symbol of the faction.
 */
export enum FactionSymbol {
  Aegis = 'AEGIS',
  Ancients = 'ANCIENTS',
  Astro = 'ASTRO',
  Cobalt = 'COBALT',
  Corsairs = 'CORSAIRS',
  Cosmic = 'COSMIC',
  Cult = 'CULT',
  Dominion = 'DOMINION',
  Echo = 'ECHO',
  Ethereal = 'ETHEREAL',
  Galactic = 'GALACTIC',
  Lords = 'LORDS',
  Obsidian = 'OBSIDIAN',
  Omega = 'OMEGA',
  Quantum = 'QUANTUM',
  Shadow = 'SHADOW',
  Solitary = 'SOLITARY',
  United = 'UNITED',
  Void = 'VOID'
}

export type Fleet = {
  __typename?: 'Fleet';
  active: Scalars['Boolean']['output'];
  allShips: Array<Ship>;
  assignments: ShipAssignmentPage;
  config: FleetConfig;
  createdAt: Scalars['DateTime']['output'];
  fleetType: FleetType;
  id: Scalars['Int']['output'];
  ships: Array<Ship>;
  system?: Maybe<System>;
  systemSymbol: Scalars['String']['output'];
  tempShips: Array<Ship>;
  updatedAt: Scalars['DateTime']['output'];
};


export type FleetAssignmentsArgs = {
  page?: InputMaybe<Scalars['Int']['input']>;
  pageSize?: InputMaybe<Scalars['Int']['input']>;
};

export type FleetBy =
  { system: Scalars['String']['input']; type?: never; }
  |  { system?: never; type: FleetType; };

export type FleetConfig = ChartingConfig | ConstructionConfig | ContractConfig | ManuelConfig | MiningConfig | ScrapingConfig | TradingConfig;

export type FleetManagerInfo = {
  __typename?: 'FleetManagerInfo';
  busy: Scalars['Boolean']['output'];
  channelState: ChannelInfo;
};

export type FleetPage = {
  __typename?: 'FleetPage';
  items: Array<Fleet>;
  page: Scalars['Int']['output'];
  pageSize?: Maybe<Scalars['Int']['output']>;
  totalCount: Scalars['Int']['output'];
};

export enum FleetType {
  Charting = 'CHARTING',
  Construction = 'CONSTRUCTION',
  Contract = 'CONTRACT',
  Manuel = 'MANUEL',
  Mining = 'MINING',
  Scrapping = 'SCRAPPING',
  Trading = 'TRADING'
}

export type FrameInfo = {
  __typename?: 'FrameInfo';
  crewRequired?: Maybe<Scalars['Int']['output']>;
  description: Scalars['String']['output'];
  fuelCapacity: Scalars['Int']['output'];
  moduleSlots: Scalars['Int']['output'];
  mountingPoints: Scalars['Int']['output'];
  name: Scalars['String']['output'];
  powerRequired?: Maybe<Scalars['Int']['output']>;
  slotsRequired?: Maybe<Scalars['Int']['output']>;
  symbol: ShipFrameSymbol;
  tradeSymbolInfo: TradeSymbolInfo;
};

export type FuelState = {
  __typename?: 'FuelState';
  capacity: Scalars['Int']['output'];
  current: Scalars['Int']['output'];
};

export enum FundStatus {
  Cancelled = 'CANCELLED',
  Reserved = 'RESERVED',
  Used = 'USED'
}

export type GateConn = {
  __typename?: 'GateConn';
  fromA: Scalars['Boolean']['output'];
  fromB: Scalars['Boolean']['output'];
  pointA?: Maybe<Waypoint>;
  pointASymbol: Scalars['String']['output'];
  pointB?: Maybe<Waypoint>;
  pointBSymbol: Scalars['String']['output'];
  underConstructionA: Scalars['Boolean']['output'];
  underConstructionB: Scalars['Boolean']['output'];
};

export type GateConnPage = {
  __typename?: 'GateConnPage';
  items: Array<GateConn>;
  page: Scalars['Int']['output'];
  pageSize?: Maybe<Scalars['Int']['output']>;
  totalCount: Scalars['Int']['output'];
};

export type IdleAssignment = {
  __typename?: 'IdleAssignment';
  controlled: Scalars['Boolean']['output'];
};

export type InputChartingConfig = {
  chartingProbeCount?: InputMaybe<Scalars['Int']['input']>;
};

export type InputConfig = {
  antimatterPrice?: InputMaybe<Scalars['Int']['input']>;
  controlActive?: InputMaybe<Scalars['Boolean']['input']>;
  controlStartSleep?: InputMaybe<Scalars['Int']['input']>;
  defaultProfit?: InputMaybe<Scalars['Int']['input']>;
  defaultPurchasePrice?: InputMaybe<Scalars['Int']['input']>;
  defaultSellPrice?: InputMaybe<Scalars['Int']['input']>;
  expand?: InputMaybe<Scalars['Boolean']['input']>;
  extraMiningTransporter?: InputMaybe<Scalars['Int']['input']>;
  fuelCost?: InputMaybe<Scalars['Int']['input']>;
  ignoreEngineeredAsteroids?: InputMaybe<Scalars['Boolean']['input']>;
  ironReserve?: InputMaybe<Scalars['Int']['input']>;
  marginPercentage?: InputMaybe<Scalars['Float']['input']>;
  marketBlacklist?: InputMaybe<Array<TradeSymbol>>;
  marketsPerShip?: InputMaybe<Scalars['Int']['input']>;
  markupPercentage?: InputMaybe<Scalars['Float']['input']>;
  maxMinersPerWaypoint?: InputMaybe<Scalars['Int']['input']>;
  maxUpdateInterval?: InputMaybe<Scalars['Int']['input']>;
  miningEjectList?: InputMaybe<Array<TradeSymbol>>;
  miningPreferList?: InputMaybe<Array<TradeSymbol>>;
  miningShipsPerWaypoint?: InputMaybe<Scalars['Int']['input']>;
  miningWaypointsPerSystem?: InputMaybe<Scalars['Int']['input']>;
  purchaseMultiplier?: InputMaybe<Scalars['Float']['input']>;
  scrapAgents?: InputMaybe<Scalars['Boolean']['input']>;
  scrapperStartSleep?: InputMaybe<Scalars['Int']['input']>;
  shipPurchaseAmount?: InputMaybe<Scalars['Int']['input']>;
  shipPurchasePercentile?: InputMaybe<Scalars['Float']['input']>;
  shipPurchaseStop?: InputMaybe<Scalars['Boolean']['input']>;
  socketAddress?: InputMaybe<Scalars['String']['input']>;
  stopAllUnstable?: InputMaybe<Scalars['Boolean']['input']>;
  tradeMode?: InputMaybe<TradeMode>;
  tradeProfitThreshold?: InputMaybe<Scalars['Int']['input']>;
  transportCapacityPerWaypoint?: InputMaybe<Scalars['Int']['input']>;
  unstableSinceTimeout?: InputMaybe<Scalars['Int']['input']>;
  updateAllSystems?: InputMaybe<Scalars['Boolean']['input']>;
};

export type InputConstructionConfig = {
  constructionShipCount?: InputMaybe<Scalars['Int']['input']>;
  constructionWaypoint?: InputMaybe<Scalars['String']['input']>;
};

export type InputContractConfig = {
  contractShipCount?: InputMaybe<Scalars['Int']['input']>;
};

export type InputFleetConfig =
  { charting: InputChartingConfig; construction?: never; contract?: never; manuel?: never; mining?: never; scraping?: never; trading?: never; }
  |  { charting?: never; construction: InputConstructionConfig; contract?: never; manuel?: never; mining?: never; scraping?: never; trading?: never; }
  |  { charting?: never; construction?: never; contract: InputContractConfig; manuel?: never; mining?: never; scraping?: never; trading?: never; }
  |  { charting?: never; construction?: never; contract?: never; manuel: InputManuelConfig; mining?: never; scraping?: never; trading?: never; }
  |  { charting?: never; construction?: never; contract?: never; manuel?: never; mining: InputMiningConfig; scraping?: never; trading?: never; }
  |  { charting?: never; construction?: never; contract?: never; manuel?: never; mining?: never; scraping: InputScrapingConfig; trading?: never; }
  |  { charting?: never; construction?: never; contract?: never; manuel?: never; mining?: never; scraping?: never; trading: InputTradingConfig; };

export type InputManuelConfig = {
  config?: InputMaybe<Scalars['String']['input']>;
};

export type InputMiningConfig = {
  ignoreEngineeredAsteroids?: InputMaybe<Scalars['Boolean']['input']>;
  minMiningCargoSpace?: InputMaybe<Scalars['Int']['input']>;
  minSiphonCargoSpace?: InputMaybe<Scalars['Int']['input']>;
  minTransporterCargoSpace?: InputMaybe<Scalars['Int']['input']>;
  minersPerWaypoint?: InputMaybe<Scalars['Int']['input']>;
  miningEjectList?: InputMaybe<Array<TradeSymbol>>;
  miningPreferList?: InputMaybe<Array<TradeSymbol>>;
  miningTransportersPerWaypoint?: InputMaybe<Scalars['Int']['input']>;
  miningWaypoints?: InputMaybe<Scalars['Int']['input']>;
  siphonersPerWaypoint?: InputMaybe<Scalars['Int']['input']>;
  stopAllUnstable?: InputMaybe<Scalars['Boolean']['input']>;
  surveyersPerWaypoint?: InputMaybe<Scalars['Int']['input']>;
  syphonWaypoints?: InputMaybe<Scalars['Int']['input']>;
  unstableSinceTimeout?: InputMaybe<Scalars['Int']['input']>;
};

export type InputScrapingConfig = {
  allowedRequests?: InputMaybe<Scalars['Int']['input']>;
  notifyOnShipyard?: InputMaybe<Scalars['Boolean']['input']>;
  shipMarketRatio?: InputMaybe<Scalars['Float']['input']>;
};

export type InputTotalChartingConfig = {
  chartingProbeCount: Scalars['Int']['input'];
};

export type InputTotalConstructionConfig = {
  constructionShipCount: Scalars['Int']['input'];
  constructionWaypoint: Scalars['String']['input'];
};

export type InputTotalContractConfig = {
  contractShipCount: Scalars['Int']['input'];
};

export type InputTotalFleetConfig =
  { charting: InputTotalChartingConfig; construction?: never; contract?: never; manuel?: never; mining?: never; scraping?: never; trading?: never; }
  |  { charting?: never; construction: InputTotalConstructionConfig; contract?: never; manuel?: never; mining?: never; scraping?: never; trading?: never; }
  |  { charting?: never; construction?: never; contract: InputTotalContractConfig; manuel?: never; mining?: never; scraping?: never; trading?: never; }
  |  { charting?: never; construction?: never; contract?: never; manuel: InputTotalManuelConfig; mining?: never; scraping?: never; trading?: never; }
  |  { charting?: never; construction?: never; contract?: never; manuel?: never; mining: InputTotalMiningConfig; scraping?: never; trading?: never; }
  |  { charting?: never; construction?: never; contract?: never; manuel?: never; mining?: never; scraping: InputTotalScrappingConfig; trading?: never; }
  |  { charting?: never; construction?: never; contract?: never; manuel?: never; mining?: never; scraping?: never; trading: InputTotalTradingConfig; };

export type InputTotalManuelConfig = {
  config: Scalars['String']['input'];
};

export type InputTotalMiningConfig = {
  ignoreEngineeredAsteroids: Scalars['Boolean']['input'];
  minMiningCargoSpace: Scalars['Int']['input'];
  minSiphonCargoSpace: Scalars['Int']['input'];
  minTransporterCargoSpace: Scalars['Int']['input'];
  minersPerWaypoint: Scalars['Int']['input'];
  miningEjectList: Array<TradeSymbol>;
  miningPreferList: Array<TradeSymbol>;
  miningTransportersPerWaypoint: Scalars['Int']['input'];
  miningWaypoints: Scalars['Int']['input'];
  siphonersPerWaypoint: Scalars['Int']['input'];
  stopAllUnstable: Scalars['Boolean']['input'];
  surveyersPerWaypoint: Scalars['Int']['input'];
  syphonWaypoints: Scalars['Int']['input'];
  unstableSinceTimeout: Scalars['Int']['input'];
};

export type InputTotalScrappingConfig = {
  allowedRequests: Scalars['Int']['input'];
  notifyOnShipyard: Scalars['Boolean']['input'];
  shipMarketRatio: Scalars['Float']['input'];
};

export type InputTotalTradingConfig = {
  marketBlacklist: Array<TradeSymbol>;
  marketPreferList: Array<TradeSymbol>;
  minCargoSpace: Scalars['Int']['input'];
  purchaseMultiplier: Scalars['Float']['input'];
  shipMarketRatio: Scalars['Float']['input'];
  tradeMode: TradeMode;
  tradeProfitThreshold: Scalars['Int']['input'];
};

export type InputTradingConfig = {
  marketBlacklist?: InputMaybe<Array<TradeSymbol>>;
  marketPreferList?: InputMaybe<Array<TradeSymbol>>;
  minCargoSpace?: InputMaybe<Scalars['Int']['input']>;
  purchaseMultiplier?: InputMaybe<Scalars['Float']['input']>;
  shipMarketRatio?: InputMaybe<Scalars['Float']['input']>;
  tradeMode?: InputMaybe<TradeMode>;
  tradeProfitThreshold?: InputMaybe<Scalars['Int']['input']>;
};

export type JumpConnection = {
  __typename?: 'JumpConnection';
  cooldownTime: Scalars['Float']['output'];
  distance: Scalars['Float']['output'];
  end?: Maybe<Waypoint>;
  endSymbol: Scalars['String']['output'];
  endSystem?: Maybe<System>;
  start?: Maybe<Waypoint>;
  startSymbol: Scalars['String']['output'];
  startSystem?: Maybe<System>;
};

export type JumpGateConnection = {
  __typename?: 'JumpGateConnection';
  createdAt: Scalars['DateTime']['output'];
  from: Scalars['String']['output'];
  id: Scalars['Int']['output'];
  systemFrom?: Maybe<System>;
  systemTo?: Maybe<System>;
  to: Scalars['String']['output'];
  updatedAt: Scalars['DateTime']['output'];
  waypointFrom?: Maybe<Waypoint>;
  waypointTo?: Maybe<Waypoint>;
};

export type JumpGateConnectionPage = {
  __typename?: 'JumpGateConnectionPage';
  items: Array<JumpGateConnection>;
  page: Scalars['Int']['output'];
  pageSize?: Maybe<Scalars['Int']['output']>;
  totalCount: Scalars['Int']['output'];
};

export type KnownAgent = {
  __typename?: 'KnownAgent';
  agent?: Maybe<Agent>;
  count: Scalars['Int']['output'];
  symbol: Scalars['String']['output'];
};

export type ManuelConfig = {
  __typename?: 'ManuelConfig';
  config: Scalars['String']['output'];
};

export type ManuelStatus = {
  __typename?: 'ManuelStatus';
  controlled: Scalars['Boolean']['output'];
};

export type MarketTrade = {
  __typename?: 'MarketTrade';
  createdAt: Scalars['DateTime']['output'];
  history: MarketTradePage;
  maps: MarketTradePage;
  marketTradeGood?: Maybe<MarketTradeGood>;
  symbol: TradeSymbol;
  tradeSymbolInfo: TradeSymbolInfo;
  type: Type;
  waypoint?: Maybe<Waypoint>;
  waypointSymbol: Scalars['String']['output'];
};


export type MarketTradeHistoryArgs = {
  page?: InputMaybe<Scalars['Int']['input']>;
  pageSize?: InputMaybe<Scalars['Int']['input']>;
};


export type MarketTradeMapsArgs = {
  page?: InputMaybe<Scalars['Int']['input']>;
  pageSize?: InputMaybe<Scalars['Int']['input']>;
};

export type MarketTradeBy =
  { system: Scalars['String']['input']; tradeSymbol?: never; waypoint?: never; }
  |  { system?: never; tradeSymbol: TradeSymbol; waypoint?: never; }
  |  { system?: never; tradeSymbol?: never; waypoint: Scalars['String']['input']; };

export type MarketTradeGood = {
  __typename?: 'MarketTradeGood';
  activity?: Maybe<ActivityLevel>;
  created: Scalars['DateTime']['output'];
  createdAt: Scalars['DateTime']['output'];
  history: MarketTradeGoodPage;
  maps: MarketTradeGoodPage;
  marketTrade?: Maybe<MarketTrade>;
  marketTransactionSummary: TransactionSummary;
  marketTransactions: MarketTransactionPage;
  purchasePrice: Scalars['Int']['output'];
  sellPrice: Scalars['Int']['output'];
  supply: SupplyLevel;
  symbol: TradeSymbol;
  tradeSymbolInfo: TradeSymbolInfo;
  tradeVolume: Scalars['Int']['output'];
  type: Type;
  waypoint?: Maybe<Waypoint>;
  waypointSymbol: Scalars['String']['output'];
};


export type MarketTradeGoodHistoryArgs = {
  page?: InputMaybe<Scalars['Int']['input']>;
  pageSize?: InputMaybe<Scalars['Int']['input']>;
};


export type MarketTradeGoodMapsArgs = {
  page?: InputMaybe<Scalars['Int']['input']>;
  pageSize?: InputMaybe<Scalars['Int']['input']>;
};


export type MarketTradeGoodMarketTransactionsArgs = {
  page?: InputMaybe<Scalars['Int']['input']>;
  pageSize?: InputMaybe<Scalars['Int']['input']>;
};

export type MarketTradeGoodBy =
  { system: Scalars['String']['input']; tradeSymbol?: never; waypoint?: never; }
  |  { system?: never; tradeSymbol: TradeSymbol; waypoint?: never; }
  |  { system?: never; tradeSymbol?: never; waypoint: Scalars['String']['input']; };

export type MarketTradeGoodPage = {
  __typename?: 'MarketTradeGoodPage';
  items: Array<MarketTradeGood>;
  page: Scalars['Int']['output'];
  pageSize?: Maybe<Scalars['Int']['output']>;
  totalCount: Scalars['Int']['output'];
};

export type MarketTradePage = {
  __typename?: 'MarketTradePage';
  items: Array<MarketTrade>;
  page: Scalars['Int']['output'];
  pageSize?: Maybe<Scalars['Int']['output']>;
  totalCount: Scalars['Int']['output'];
};

export type MarketTransaction = {
  __typename?: 'MarketTransaction';
  constructionShipment?: Maybe<ConstructionShipment>;
  construction_shipment_id?: Maybe<Scalars['Int']['output']>;
  contract?: Maybe<Contract>;
  /**
   * The reason for the transaction.
   * pub reason: TransactionReason,
   */
  contract_id?: Maybe<Scalars['String']['output']>;
  id: Scalars['Int']['output'];
  marketTradeGood?: Maybe<MarketTradeGood>;
  miningWaypoint?: Maybe<Waypoint>;
  mining_waypoint_symbol?: Maybe<Scalars['String']['output']>;
  /** The price per unit of the transaction. */
  pricePerUnit: Scalars['Int']['output'];
  ship?: Maybe<Ship>;
  /** The symbol of the ship that made the transaction. */
  shipSymbol: Scalars['String']['output'];
  /** The timestamp of the transaction. */
  timestamp: Scalars['DateTime']['output'];
  /** The total price of the transaction. */
  totalPrice: Scalars['Int']['output'];
  tradeRoute?: Maybe<TradeRoute>;
  /** The symbol of the trade good. */
  tradeSymbol: TradeSymbol;
  tradeSymbolInfo: TradeSymbolInfo;
  trade_route_id?: Maybe<Scalars['Int']['output']>;
  /** The type of transaction. */
  type: MarketTransactionType;
  /** The number of units of the transaction. */
  units: Scalars['Int']['output'];
  waypoint?: Maybe<Waypoint>;
  /** The symbol of the waypoint. */
  waypointSymbol: Scalars['String']['output'];
};

export type MarketTransactionBy =
  { construction: Scalars['Int']['input']; contract?: never; mining?: never; shipSymbol?: never; system?: never; tradeRoute?: never; tradeSymbol?: never; type?: never; waypoint?: never; }
  |  { construction?: never; contract: Scalars['String']['input']; mining?: never; shipSymbol?: never; system?: never; tradeRoute?: never; tradeSymbol?: never; type?: never; waypoint?: never; }
  |  { construction?: never; contract?: never; mining: Scalars['String']['input']; shipSymbol?: never; system?: never; tradeRoute?: never; tradeSymbol?: never; type?: never; waypoint?: never; }
  |  { construction?: never; contract?: never; mining?: never; shipSymbol: Scalars['String']['input']; system?: never; tradeRoute?: never; tradeSymbol?: never; type?: never; waypoint?: never; }
  |  { construction?: never; contract?: never; mining?: never; shipSymbol?: never; system: Scalars['String']['input']; tradeRoute?: never; tradeSymbol?: never; type?: never; waypoint?: never; }
  |  { construction?: never; contract?: never; mining?: never; shipSymbol?: never; system?: never; tradeRoute: Scalars['Int']['input']; tradeSymbol?: never; type?: never; waypoint?: never; }
  |  { construction?: never; contract?: never; mining?: never; shipSymbol?: never; system?: never; tradeRoute?: never; tradeSymbol: TradeSymbol; type?: never; waypoint?: never; }
  |  { construction?: never; contract?: never; mining?: never; shipSymbol?: never; system?: never; tradeRoute?: never; tradeSymbol?: never; type: MarketTransactionType; waypoint?: never; }
  |  { construction?: never; contract?: never; mining?: never; shipSymbol?: never; system?: never; tradeRoute?: never; tradeSymbol?: never; type?: never; waypoint: Scalars['String']['input']; };

export type MarketTransactionPage = {
  __typename?: 'MarketTransactionPage';
  items: Array<MarketTransaction>;
  page: Scalars['Int']['output'];
  pageSize?: Maybe<Scalars['Int']['output']>;
  totalCount: Scalars['Int']['output'];
};

/** The type of transaction. */
export enum MarketTransactionType {
  Purchase = 'PURCHASE',
  Sell = 'SELL'
}

export type MiningAssignment = {
  __typename?: 'MiningAssignment';
  assignedShips: Array<AssignedShip>;
  lastUpdated: Scalars['DateTime']['output'];
  waypointSymbol: Scalars['String']['output'];
};

export type MiningConfig = {
  __typename?: 'MiningConfig';
  ignoreEngineeredAsteroids: Scalars['Boolean']['output'];
  minMiningCargoSpace: Scalars['Int']['output'];
  minSiphonCargoSpace: Scalars['Int']['output'];
  minTransporterCargoSpace: Scalars['Int']['output'];
  minersPerWaypoint: Scalars['Int']['output'];
  miningEjectList: Array<TradeSymbol>;
  miningPreferList: Array<TradeSymbol>;
  miningTransportersPerWaypoint: Scalars['Int']['output'];
  miningWaypoints: Scalars['Int']['output'];
  siphonersPerWaypoint: Scalars['Int']['output'];
  stopAllUnstable: Scalars['Boolean']['output'];
  surveyersPerWaypoint: Scalars['Int']['output'];
  syphonWaypoints: Scalars['Int']['output'];
  unstableSinceTimeout: Scalars['Int']['output'];
};

export type MiningManagerInfo = {
  __typename?: 'MiningManagerInfo';
  busy: Scalars['Boolean']['output'];
  channelState: ChannelInfo;
  getAssignments: Array<MiningAssignment>;
};

export type MiningShipAssignment = ExtractorAssignment | IdleAssignment | SiphonerAssignment | SurveyorAssignment | TransporterAssignment | UselessAssignment;

export type MiningStatus = {
  __typename?: 'MiningStatus';
  assignment: MiningShipAssignment;
};

export type ModuleInfo = {
  __typename?: 'ModuleInfo';
  capacity?: Maybe<Scalars['Int']['output']>;
  crewRequired?: Maybe<Scalars['Int']['output']>;
  description: Scalars['String']['output'];
  name: Scalars['String']['output'];
  powerRequired?: Maybe<Scalars['Int']['output']>;
  range?: Maybe<Scalars['Int']['output']>;
  slotsRequired?: Maybe<Scalars['Int']['output']>;
  symbol: ShipModuleSymbol;
  tradeSymbolInfo: TradeSymbolInfo;
};

export type Modules = {
  __typename?: 'Modules';
  moduleInfos: Array<ModuleInfo>;
  modules: Array<ShipModuleSymbol>;
};

export type MountInfo = {
  __typename?: 'MountInfo';
  crewRequired?: Maybe<Scalars['Int']['output']>;
  deposits?: Maybe<Array<TradeSymbol>>;
  description: Scalars['String']['output'];
  name: Scalars['String']['output'];
  powerRequired?: Maybe<Scalars['Int']['output']>;
  slotsRequired?: Maybe<Scalars['Int']['output']>;
  strength?: Maybe<Scalars['Int']['output']>;
  symbol: ShipMountSymbol;
  tradeSymbolInfo: TradeSymbolInfo;
};

export type Mounts = {
  __typename?: 'Mounts';
  mountInfos: Array<MountInfo>;
  mounts: Array<ShipMountSymbol>;
};

export type MutationRoot = {
  __typename?: 'MutationRoot';
  /** Add a new fleet for a given system. Returns the created DB fleet. */
  addFleet: DbFleet;
  /** Blacklist a system from population */
  blacklistSystem: Scalars['Boolean']['output'];
  /** Remove a system from the blacklist */
  deblacklistSystem: Scalars['Boolean']['output'];
  /** Edit the in-memory config (and return the updated config). Fields that are None are left unchanged. */
  editConfig: Config;
  /** Edit basic fleet attributes and configuration. Fields that are None are left unchanged. */
  editFleet: DbFleet;
  /** Force assign a ship a new assignment from the fleet manager. */
  forceAssignShip: ShipInfo;
  /** Pause a ship (set active=false in ship_info) */
  pauseShip: Scalars['Boolean']['output'];
  /** Trigger a regeneration of fleet assignments. This will ask the FleetManager to rebuild assignments. */
  regenerateFleetAssignments: Scalars['Boolean']['output'];
  /** Remove a fleet. */
  removeFleet: Scalars['Boolean']['output'];
  /** Repopulate a system with fleets */
  repopulateSystemWithFleets: Scalars['Boolean']['output'];
  /** Repopulate systems connecting to a jump gate with fleets */
  repopulateSystemsWithFleetsFromJumpGate: Scalars['Boolean']['output'];
  /** Resume a ship (set active=true in ship_info) */
  resumeShip: Scalars['Boolean']['output'];
};


export type MutationRootAddFleetArgs = {
  active: Scalars['Boolean']['input'];
  config: InputTotalFleetConfig;
  systemSymbol: Scalars['String']['input'];
};


export type MutationRootBlacklistSystemArgs = {
  system: Scalars['String']['input'];
};


export type MutationRootDeblacklistSystemArgs = {
  system: Scalars['String']['input'];
};


export type MutationRootEditConfigArgs = {
  input: InputConfig;
};


export type MutationRootEditFleetArgs = {
  active?: InputMaybe<Scalars['Boolean']['input']>;
  config?: InputMaybe<InputFleetConfig>;
  id: Scalars['Int']['input'];
  systemSymbol?: InputMaybe<Scalars['String']['input']>;
};


export type MutationRootForceAssignShipArgs = {
  assignmentId: Scalars['Int']['input'];
  shipSymbol: Scalars['String']['input'];
  temp: Scalars['Boolean']['input'];
};


export type MutationRootPauseShipArgs = {
  shipSymbol: Scalars['String']['input'];
};


export type MutationRootRegenerateFleetAssignmentsArgs = {
  by?: InputMaybe<RegenFleetBy>;
};


export type MutationRootRemoveFleetArgs = {
  id: Scalars['Int']['input'];
};


export type MutationRootRepopulateSystemWithFleetsArgs = {
  system: Scalars['String']['input'];
};


export type MutationRootRepopulateSystemsWithFleetsFromJumpGateArgs = {
  jumpGate: Scalars['String']['input'];
};


export type MutationRootResumeShipArgs = {
  shipSymbol: Scalars['String']['input'];
};

export type NavigateConnection = {
  __typename?: 'NavigateConnection';
  distance: Scalars['Float']['output'];
  end?: Maybe<Waypoint>;
  endIsMarketplace: Scalars['Boolean']['output'];
  endSymbol: Scalars['String']['output'];
  endSystem?: Maybe<System>;
  navMode: ShipNavFlightMode;
  refuel: Refuel;
  start?: Maybe<Waypoint>;
  startIsMarketplace: Scalars['Boolean']['output'];
  startSymbol: Scalars['String']['output'];
  startSystem?: Maybe<System>;
  travelTime: Scalars['Float']['output'];
};

export type NavigationState = {
  __typename?: 'NavigationState';
  autoPilot?: Maybe<AutopilotState>;
  flightMode: ShipNavFlightMode;
  route: RouteState;
  status: ShipNavStatus;
  system?: Maybe<System>;
  systemSymbol: Scalars['String']['output'];
  waypoint?: Maybe<Waypoint>;
  waypointSymbol: Scalars['String']['output'];
};

export type QueryRoot = {
  __typename?: 'QueryRoot';
  agent: Agent;
  agentHistory: Array<Agent>;
  agents: Array<Agent>;
  apiCounts: Scalars['Int']['output'];
  budget: BudgetInfo;
  chartManager: ChartManagerInfo;
  chartTransactions: ChartTransactionPage;
  config: Config;
  constructionManager: ConstructionManagerInfo;
  constructionMaterials: ConstructionMaterialPage;
  constructionShipments: ConstructionShipmentPage;
  contract: Contract;
  contractDeliveries: ContractDeliveryPage;
  contractManager: ContractManagerInfo;
  contractShipments: ContractShipmentPage;
  contracts: ContractPage;
  extraction: Extraction;
  extractions: ExtractionPage;
  fleetManager: FleetManagerInfo;
  fleets: FleetPage;
  jumpConnections: GateConnPage;
  jumpGateConnections: JumpGateConnectionPage;
  marketTradeGoods: MarketTradeGoodPage;
  marketTrades: MarketTradePage;
  marketTransactions: MarketTransactionPage;
  miningManager: MiningManagerInfo;
  repairTransactions: RepairTransactionPage;
  reservedFunds: ReservedFundPage;
  runInfo: RunInfo;
  scrapTransactions: ScrapTransactionPage;
  scrappingManager: ScrappingManagerInfo;
  ship: Ship;
  shipAssignments: ShipAssignmentPage;
  shipInfo: ShipInfo;
  shipInfos: ShipInfoPage;
  shipModificationTransactions: ShipModificationTransactionPage;
  shipRoutes: RoutePage;
  shipStates: ShipStatePage;
  ships: Array<Ship>;
  shipyard: Shipyard;
  shipyardShips: ShipyardShipPage;
  shipyardTransactions: ShipyardTransactionPage;
  shipyards: ShipyardPage;
  survey: Survey;
  surveys: SurveyPage;
  system: System;
  systems: SystemPage;
  tradeManager: TradeManagerInfo;
  tradeRoute: TradeRoute;
  tradeRoutes: TradeRoutePage;
  tradeSymbolInfos: Array<TradeSymbolInfo>;
  waypoint: Waypoint;
  waypoints: WaypointPage;
};


export type QueryRootAgentArgs = {
  symbol: Scalars['String']['input'];
};


export type QueryRootAgentHistoryArgs = {
  symbol: Scalars['String']['input'];
};


export type QueryRootChartTransactionsArgs = {
  page?: InputMaybe<Scalars['Int']['input']>;
  pageSize?: InputMaybe<Scalars['Int']['input']>;
  shipSymbol?: InputMaybe<Scalars['String']['input']>;
};


export type QueryRootConstructionMaterialsArgs = {
  by?: InputMaybe<ConstructionMaterialBy>;
  page?: InputMaybe<Scalars['Int']['input']>;
  pageSize?: InputMaybe<Scalars['Int']['input']>;
};


export type QueryRootConstructionShipmentsArgs = {
  by?: InputMaybe<ConstructionShipmentBy>;
  page?: InputMaybe<Scalars['Int']['input']>;
  pageSize?: InputMaybe<Scalars['Int']['input']>;
};


export type QueryRootContractArgs = {
  symbol: Scalars['String']['input'];
};


export type QueryRootContractDeliveriesArgs = {
  by?: InputMaybe<ContractDeliveryBy>;
  page?: InputMaybe<Scalars['Int']['input']>;
  pageSize?: InputMaybe<Scalars['Int']['input']>;
};


export type QueryRootContractShipmentsArgs = {
  by?: InputMaybe<ContractShipmentBy>;
  page?: InputMaybe<Scalars['Int']['input']>;
  pageSize?: InputMaybe<Scalars['Int']['input']>;
};


export type QueryRootContractsArgs = {
  by?: InputMaybe<ContractBy>;
  page?: InputMaybe<Scalars['Int']['input']>;
  pageSize?: InputMaybe<Scalars['Int']['input']>;
};


export type QueryRootExtractionArgs = {
  symbol: Scalars['Int']['input'];
};


export type QueryRootExtractionsArgs = {
  by?: InputMaybe<ExtractionBy>;
  page?: InputMaybe<Scalars['Int']['input']>;
  pageSize?: InputMaybe<Scalars['Int']['input']>;
};


export type QueryRootFleetsArgs = {
  by?: InputMaybe<FleetBy>;
  page?: InputMaybe<Scalars['Int']['input']>;
  pageSize?: InputMaybe<Scalars['Int']['input']>;
};


export type QueryRootJumpConnectionsArgs = {
  page?: InputMaybe<Scalars['Int']['input']>;
  pageSize?: InputMaybe<Scalars['Int']['input']>;
};


export type QueryRootJumpGateConnectionsArgs = {
  from?: InputMaybe<Scalars['String']['input']>;
  page?: InputMaybe<Scalars['Int']['input']>;
  pageSize?: InputMaybe<Scalars['Int']['input']>;
};


export type QueryRootMarketTradeGoodsArgs = {
  by?: InputMaybe<MarketTradeGoodBy>;
  page?: InputMaybe<Scalars['Int']['input']>;
  pageSize?: InputMaybe<Scalars['Int']['input']>;
};


export type QueryRootMarketTradesArgs = {
  by?: InputMaybe<MarketTradeBy>;
  page?: InputMaybe<Scalars['Int']['input']>;
  pageSize?: InputMaybe<Scalars['Int']['input']>;
};


export type QueryRootMarketTransactionsArgs = {
  by?: InputMaybe<MarketTransactionBy>;
  page?: InputMaybe<Scalars['Int']['input']>;
  pageSize?: InputMaybe<Scalars['Int']['input']>;
};


export type QueryRootRepairTransactionsArgs = {
  page?: InputMaybe<Scalars['Int']['input']>;
  pageSize?: InputMaybe<Scalars['Int']['input']>;
};


export type QueryRootReservedFundsArgs = {
  page?: InputMaybe<Scalars['Int']['input']>;
  pageSize?: InputMaybe<Scalars['Int']['input']>;
};


export type QueryRootScrapTransactionsArgs = {
  page?: InputMaybe<Scalars['Int']['input']>;
  pageSize?: InputMaybe<Scalars['Int']['input']>;
};


export type QueryRootShipArgs = {
  symbol: Scalars['String']['input'];
};


export type QueryRootShipAssignmentsArgs = {
  by?: InputMaybe<ShipAssignmentBy>;
  page?: InputMaybe<Scalars['Int']['input']>;
  pageSize?: InputMaybe<Scalars['Int']['input']>;
};


export type QueryRootShipInfoArgs = {
  symbol: Scalars['String']['input'];
};


export type QueryRootShipInfosArgs = {
  page?: InputMaybe<Scalars['Int']['input']>;
  pageSize?: InputMaybe<Scalars['Int']['input']>;
};


export type QueryRootShipModificationTransactionsArgs = {
  page?: InputMaybe<Scalars['Int']['input']>;
  pageSize?: InputMaybe<Scalars['Int']['input']>;
};


export type QueryRootShipRoutesArgs = {
  page?: InputMaybe<Scalars['Int']['input']>;
  pageSize?: InputMaybe<Scalars['Int']['input']>;
};


export type QueryRootShipStatesArgs = {
  by?: InputMaybe<ShipStateBy>;
  page?: InputMaybe<Scalars['Int']['input']>;
  pageSize?: InputMaybe<Scalars['Int']['input']>;
};


export type QueryRootShipyardArgs = {
  symbol: Scalars['String']['input'];
};


export type QueryRootShipyardShipsArgs = {
  by?: InputMaybe<ShipyardShipBy>;
  page?: InputMaybe<Scalars['Int']['input']>;
  pageSize?: InputMaybe<Scalars['Int']['input']>;
};


export type QueryRootShipyardTransactionsArgs = {
  by?: InputMaybe<ShipyardTransactionBy>;
  page?: InputMaybe<Scalars['Int']['input']>;
  pageSize?: InputMaybe<Scalars['Int']['input']>;
};


export type QueryRootShipyardsArgs = {
  page?: InputMaybe<Scalars['Int']['input']>;
  pageSize?: InputMaybe<Scalars['Int']['input']>;
};


export type QueryRootSurveyArgs = {
  signature: Scalars['String']['input'];
};


export type QueryRootSurveysArgs = {
  by?: InputMaybe<SurveyBy>;
  page?: InputMaybe<Scalars['Int']['input']>;
  pageSize?: InputMaybe<Scalars['Int']['input']>;
};


export type QueryRootSystemArgs = {
  symbol: Scalars['String']['input'];
};


export type QueryRootSystemsArgs = {
  onlyWithFleetsOrShips?: InputMaybe<Scalars['Boolean']['input']>;
  page?: InputMaybe<Scalars['Int']['input']>;
  pageSize?: InputMaybe<Scalars['Int']['input']>;
};


export type QueryRootTradeRouteArgs = {
  routeId: Scalars['Int']['input'];
};


export type QueryRootTradeRoutesArgs = {
  page?: InputMaybe<Scalars['Int']['input']>;
  pageSize?: InputMaybe<Scalars['Int']['input']>;
};


export type QueryRootWaypointArgs = {
  symbol: Scalars['String']['input'];
};


export type QueryRootWaypointsArgs = {
  page?: InputMaybe<Scalars['Int']['input']>;
  pageSize?: InputMaybe<Scalars['Int']['input']>;
};

export type ReactorInfo = {
  __typename?: 'ReactorInfo';
  crewRequired?: Maybe<Scalars['Int']['output']>;
  description: Scalars['String']['output'];
  name: Scalars['String']['output'];
  powerOutput: Scalars['Int']['output'];
  powerRequired?: Maybe<Scalars['Int']['output']>;
  slotsRequired?: Maybe<Scalars['Int']['output']>;
  symbol: ShipReactorSymbol;
  tradeSymbolInfo: TradeSymbolInfo;
};

export type Refuel = {
  __typename?: 'Refuel';
  fuelNeeded: Scalars['Int']['output'];
  fuelRequired: Scalars['Int']['output'];
  startIsMarketplace: Scalars['Boolean']['output'];
};

export type RegenFleetBy =
  { fleet: Scalars['Int']['input']; system?: never; }
  |  { fleet?: never; system: Scalars['String']['input']; };

export type RepairTransaction = {
  __typename?: 'RepairTransaction';
  id: Scalars['Int']['output'];
  ship?: Maybe<Ship>;
  shipSymbol: Scalars['String']['output'];
  timestamp: Scalars['DateTime']['output'];
  totalPrice: Scalars['Int']['output'];
  waypoint?: Maybe<Waypoint>;
  waypointSymbol: Scalars['String']['output'];
};

export type RepairTransactionPage = {
  __typename?: 'RepairTransactionPage';
  items: Array<RepairTransaction>;
  page: Scalars['Int']['output'];
  pageSize?: Maybe<Scalars['Int']['output']>;
  totalCount: Scalars['Int']['output'];
};

export type ReservedFund = {
  __typename?: 'ReservedFund';
  actualAmount: Scalars['Int']['output'];
  amount: Scalars['Int']['output'];
  constructionShipment: ConstructionShipmentPage;
  contract: ContractPage;
  createdAt: Scalars['DateTime']['output'];
  id: Scalars['Int']['output'];
  status: FundStatus;
  tradeRoute: TradeRoutePage;
  updatedAt: Scalars['DateTime']['output'];
};


export type ReservedFundConstructionShipmentArgs = {
  page?: InputMaybe<Scalars['Int']['input']>;
  pageSize?: InputMaybe<Scalars['Int']['input']>;
};


export type ReservedFundContractArgs = {
  page?: InputMaybe<Scalars['Int']['input']>;
  pageSize?: InputMaybe<Scalars['Int']['input']>;
};


export type ReservedFundTradeRouteArgs = {
  page?: InputMaybe<Scalars['Int']['input']>;
  pageSize?: InputMaybe<Scalars['Int']['input']>;
};

export type ReservedFundPage = {
  __typename?: 'ReservedFundPage';
  items: Array<ReservedFund>;
  page: Scalars['Int']['output'];
  pageSize?: Maybe<Scalars['Int']['output']>;
  totalCount: Scalars['Int']['output'];
};

export type Route = {
  __typename?: 'Route';
  createdAt: Scalars['DateTime']['output'];
  distance: Scalars['Float']['output'];
  from: Scalars['String']['output'];
  fuelCost: Scalars['Int']['output'];
  id: Scalars['Int']['output'];
  navMode: Scalars['String']['output'];
  ship?: Maybe<Ship>;
  shipInfoAfter?: Maybe<Scalars['Int']['output']>;
  shipInfoBefore?: Maybe<Scalars['Int']['output']>;
  shipStateAfter?: Maybe<ShipState>;
  shipStateBefore?: Maybe<ShipState>;
  shipSymbol: Scalars['String']['output'];
  to: Scalars['String']['output'];
  travelTime: Scalars['Float']['output'];
  waypointFrom?: Maybe<Waypoint>;
  waypointTo?: Maybe<Waypoint>;
};

export type RoutePage = {
  __typename?: 'RoutePage';
  items: Array<Route>;
  page: Scalars['Int']['output'];
  pageSize?: Maybe<Scalars['Int']['output']>;
  totalCount: Scalars['Int']['output'];
};

export type RouteState = {
  __typename?: 'RouteState';
  arrival: Scalars['DateTime']['output'];
  departureTime: Scalars['DateTime']['output'];
  destinationSymbol: Scalars['String']['output'];
  destinationSystem?: Maybe<System>;
  destinationSystemSymbol: Scalars['String']['output'];
  destinationWaypoint?: Maybe<Waypoint>;
  originSymbol: Scalars['String']['output'];
  originSystem?: Maybe<System>;
  originSystemSymbol: Scalars['String']['output'];
  originWaypoint?: Maybe<Waypoint>;
};

export type RunInfo = {
  __typename?: 'RunInfo';
  agent?: Maybe<Agent>;
  agentSymbol: Scalars['String']['output'];
  headquarters: Scalars['String']['output'];
  headquartersSystem?: Maybe<System>;
  headquartersWaypoint?: Maybe<Waypoint>;
  nextResetDate: Scalars['DateTime']['output'];
  resetDate: Scalars['DateTime']['output'];
  startingFaction: FactionSymbol;
  version: Scalars['String']['output'];
};

export type ScrapInfo = {
  __typename?: 'ScrapInfo';
  date: Scalars['DateTime']['output'];
  waypointSymbol: Scalars['String']['output'];
};

export type ScrapTransaction = {
  __typename?: 'ScrapTransaction';
  id: Scalars['Int']['output'];
  ship?: Maybe<Ship>;
  shipSymbol: Scalars['String']['output'];
  timestamp: Scalars['DateTime']['output'];
  totalPrice: Scalars['Int']['output'];
  waypoint?: Maybe<Waypoint>;
  waypointSymbol: Scalars['String']['output'];
};

export type ScrapTransactionPage = {
  __typename?: 'ScrapTransactionPage';
  items: Array<ScrapTransaction>;
  page: Scalars['Int']['output'];
  pageSize?: Maybe<Scalars['Int']['output']>;
  totalCount: Scalars['Int']['output'];
};

export type ScraperStatus = {
  __typename?: 'ScraperStatus';
  cycle?: Maybe<Scalars['Int']['output']>;
  scrapDate?: Maybe<Scalars['DateTime']['output']>;
  waitingForManager: Scalars['Boolean']['output'];
  waypoint?: Maybe<Waypoint>;
  waypointSymbol?: Maybe<Scalars['String']['output']>;
};

export type ScrapingConfig = {
  __typename?: 'ScrapingConfig';
  allowedRequests: Scalars['Int']['output'];
  notifyOnShipyard: Scalars['Boolean']['output'];
  shipMarketRatio: Scalars['Float']['output'];
};

export type ScrappingManagerInfo = {
  __typename?: 'ScrappingManagerInfo';
  busy: Scalars['Boolean']['output'];
  channelState: ChannelInfo;
  possibleScraps: Array<ScrapInfo>;
};


export type ScrappingManagerInfoPossibleScrapsArgs = {
  shipSymbol: Scalars['String']['input'];
};

export type Ship = {
  __typename?: 'Ship';
  cargo: CargoState;
  chartTransactions: ChartTransactionPage;
  conditions: ConditionState;
  constructionShipments: ConstructionShipmentPage;
  contractShipments: ContractShipmentPage;
  cooldown?: Maybe<Scalars['Int']['output']>;
  cooldownExpiration?: Maybe<Scalars['DateTime']['output']>;
  displayName: Scalars['String']['output'];
  engine: ShipEngineSymbol;
  engineInfo: EngineInfo;
  engineSpeed: Scalars['Int']['output'];
  extractions: ExtractionPage;
  frame: ShipFrameSymbol;
  frameInfo: FrameInfo;
  fuel: FuelState;
  marketTransactionSummary: TransactionSummary;
  marketTransactions: MarketTransactionPage;
  modules: Modules;
  mounts: Mounts;
  nav: NavigationState;
  possibleScraps: Array<ScrapInfo>;
  purchaseId?: Maybe<Scalars['Int']['output']>;
  purchaseTransaction?: Maybe<ShipyardTransaction>;
  reactor: ShipReactorSymbol;
  reactorInfo: ReactorInfo;
  registrationRole: ShipRole;
  repairTransactions: RepairTransactionPage;
  routes: RoutePage;
  scrapTransactions: ScrapTransactionPage;
  shipJumps: ShipJumpPage;
  shipModificationTransactions: ShipModificationTransactionPage;
  shipStates: ShipStatePage;
  status: ShipStatus;
  surveys: SurveyPage;
  symbol: Scalars['String']['output'];
  tradeRoutes: TradeRoutePage;
};


export type ShipChartTransactionsArgs = {
  page?: InputMaybe<Scalars['Int']['input']>;
  pageSize?: InputMaybe<Scalars['Int']['input']>;
};


export type ShipConstructionShipmentsArgs = {
  page?: InputMaybe<Scalars['Int']['input']>;
  pageSize?: InputMaybe<Scalars['Int']['input']>;
};


export type ShipContractShipmentsArgs = {
  page?: InputMaybe<Scalars['Int']['input']>;
  pageSize?: InputMaybe<Scalars['Int']['input']>;
};


export type ShipExtractionsArgs = {
  page?: InputMaybe<Scalars['Int']['input']>;
  pageSize?: InputMaybe<Scalars['Int']['input']>;
};


export type ShipMarketTransactionsArgs = {
  page?: InputMaybe<Scalars['Int']['input']>;
  pageSize?: InputMaybe<Scalars['Int']['input']>;
};


export type ShipRepairTransactionsArgs = {
  page?: InputMaybe<Scalars['Int']['input']>;
  pageSize?: InputMaybe<Scalars['Int']['input']>;
};


export type ShipRoutesArgs = {
  page?: InputMaybe<Scalars['Int']['input']>;
  pageSize?: InputMaybe<Scalars['Int']['input']>;
};


export type ShipScrapTransactionsArgs = {
  page?: InputMaybe<Scalars['Int']['input']>;
  pageSize?: InputMaybe<Scalars['Int']['input']>;
};


export type ShipShipJumpsArgs = {
  page?: InputMaybe<Scalars['Int']['input']>;
  pageSize?: InputMaybe<Scalars['Int']['input']>;
};


export type ShipShipModificationTransactionsArgs = {
  page?: InputMaybe<Scalars['Int']['input']>;
  pageSize?: InputMaybe<Scalars['Int']['input']>;
};


export type ShipShipStatesArgs = {
  page?: InputMaybe<Scalars['Int']['input']>;
  pageSize?: InputMaybe<Scalars['Int']['input']>;
};


export type ShipSurveysArgs = {
  page?: InputMaybe<Scalars['Int']['input']>;
  pageSize?: InputMaybe<Scalars['Int']['input']>;
};


export type ShipTradeRoutesArgs = {
  page?: InputMaybe<Scalars['Int']['input']>;
  pageSize?: InputMaybe<Scalars['Int']['input']>;
};

export type ShipAssignment = {
  __typename?: 'ShipAssignment';
  cargoMin: Scalars['Int']['output'];
  creditsThreshold: Scalars['Int']['output'];
  disabled: Scalars['Boolean']['output'];
  extractor: Scalars['Boolean']['output'];
  fleet?: Maybe<Fleet>;
  fleetId: Scalars['Int']['output'];
  id: Scalars['Int']['output'];
  maxPurchasePrice: Scalars['Int']['output'];
  permanentShip?: Maybe<Ship>;
  priority: Scalars['Int']['output'];
  rangeMin: Scalars['Int']['output'];
  ship?: Maybe<Ship>;
  siphon: Scalars['Boolean']['output'];
  survey: Scalars['Boolean']['output'];
  tempShip?: Maybe<Ship>;
  warpDrive: Scalars['Boolean']['output'];
};

export type ShipAssignmentBy =
  { fleet: Scalars['Int']['input']; open?: never; }
  |  { fleet?: never; open: Scalars['Boolean']['input']; };

export type ShipAssignmentPage = {
  __typename?: 'ShipAssignmentPage';
  items: Array<ShipAssignment>;
  page: Scalars['Int']['output'];
  pageSize?: Maybe<Scalars['Int']['output']>;
  totalCount: Scalars['Int']['output'];
};

/** The symbol of the engine. */
export enum ShipEngineSymbol {
  HyperDriveI = 'HYPER_DRIVE_I',
  ImpulseDriveI = 'IMPULSE_DRIVE_I',
  IonDriveI = 'ION_DRIVE_I',
  IonDriveIi = 'ION_DRIVE_II'
}

/** Symbol of the frame. */
export enum ShipFrameSymbol {
  BulkFreighter = 'BULK_FREIGHTER',
  Carrier = 'CARRIER',
  Cruiser = 'CRUISER',
  Destroyer = 'DESTROYER',
  Drone = 'DRONE',
  Explorer = 'EXPLORER',
  Fighter = 'FIGHTER',
  Frigate = 'FRIGATE',
  HeavyFreighter = 'HEAVY_FREIGHTER',
  Interceptor = 'INTERCEPTOR',
  LightFreighter = 'LIGHT_FREIGHTER',
  Miner = 'MINER',
  Probe = 'PROBE',
  Racer = 'RACER',
  Shuttle = 'SHUTTLE',
  Transport = 'TRANSPORT'
}

export type ShipInfo = {
  __typename?: 'ShipInfo';
  active: Scalars['Boolean']['output'];
  assignment?: Maybe<ShipAssignment>;
  assignmentId?: Maybe<Scalars['Int']['output']>;
  displayName: Scalars['String']['output'];
  purchaseId?: Maybe<Scalars['Int']['output']>;
  purchaseTransaction?: Maybe<ShipyardTransaction>;
  ship?: Maybe<Ship>;
  symbol: Scalars['String']['output'];
  tempAssignment?: Maybe<ShipAssignment>;
  tempAssignmentId?: Maybe<Scalars['Int']['output']>;
};

export type ShipInfoPage = {
  __typename?: 'ShipInfoPage';
  items: Array<ShipInfo>;
  page: Scalars['Int']['output'];
  pageSize?: Maybe<Scalars['Int']['output']>;
  totalCount: Scalars['Int']['output'];
};

export type ShipJump = {
  __typename?: 'ShipJump';
  distance: Scalars['Int']['output'];
  from: Scalars['String']['output'];
  id: Scalars['Int']['output'];
  ship?: Maybe<Ship>;
  shipAfter: Scalars['Int']['output'];
  shipBefore: Scalars['Int']['output'];
  shipStateAfter?: Maybe<ShipState>;
  shipStateBefore?: Maybe<ShipState>;
  shipSymbol: Scalars['String']['output'];
  to: Scalars['String']['output'];
  waypointFrom?: Maybe<Waypoint>;
  waypointTo?: Maybe<Waypoint>;
};

export type ShipJumpPage = {
  __typename?: 'ShipJumpPage';
  items: Array<ShipJump>;
  page: Scalars['Int']['output'];
  pageSize?: Maybe<Scalars['Int']['output']>;
  totalCount: Scalars['Int']['output'];
};

export type ShipModificationTransaction = {
  __typename?: 'ShipModificationTransaction';
  id: Scalars['Int']['output'];
  ship?: Maybe<Ship>;
  shipSymbol: Scalars['String']['output'];
  timestamp: Scalars['DateTime']['output'];
  totalPrice: Scalars['Int']['output'];
  tradeSymbol: TradeSymbol;
  tradeSymbolInfo: TradeSymbolInfo;
  waypoint?: Maybe<Waypoint>;
  waypointSymbol: Scalars['String']['output'];
};

export type ShipModificationTransactionPage = {
  __typename?: 'ShipModificationTransactionPage';
  items: Array<ShipModificationTransaction>;
  page: Scalars['Int']['output'];
  pageSize?: Maybe<Scalars['Int']['output']>;
  totalCount: Scalars['Int']['output'];
};

/** The symbol of the module. */
export enum ShipModuleSymbol {
  CargoHoldI = 'CARGO_HOLD_I',
  CargoHoldIi = 'CARGO_HOLD_II',
  CargoHoldIii = 'CARGO_HOLD_III',
  CrewQuartersI = 'CREW_QUARTERS_I',
  EnvoyQuartersI = 'ENVOY_QUARTERS_I',
  FuelRefineryI = 'FUEL_REFINERY_I',
  GasProcessorI = 'GAS_PROCESSOR_I',
  JumpDriveI = 'JUMP_DRIVE_I',
  JumpDriveIi = 'JUMP_DRIVE_II',
  JumpDriveIii = 'JUMP_DRIVE_III',
  MicroRefineryI = 'MICRO_REFINERY_I',
  MineralProcessorI = 'MINERAL_PROCESSOR_I',
  OreRefineryI = 'ORE_REFINERY_I',
  PassengerCabinI = 'PASSENGER_CABIN_I',
  ScienceLabI = 'SCIENCE_LAB_I',
  ShieldGeneratorI = 'SHIELD_GENERATOR_I',
  ShieldGeneratorIi = 'SHIELD_GENERATOR_II',
  WarpDriveI = 'WARP_DRIVE_I',
  WarpDriveIi = 'WARP_DRIVE_II',
  WarpDriveIii = 'WARP_DRIVE_III'
}

/** Symbol of this mount. */
export enum ShipMountSymbol {
  GasSiphonI = 'GAS_SIPHON_I',
  GasSiphonIi = 'GAS_SIPHON_II',
  GasSiphonIii = 'GAS_SIPHON_III',
  LaserCannonI = 'LASER_CANNON_I',
  MiningLaserI = 'MINING_LASER_I',
  MiningLaserIi = 'MINING_LASER_II',
  MiningLaserIii = 'MINING_LASER_III',
  MissileLauncherI = 'MISSILE_LAUNCHER_I',
  SensorArrayI = 'SENSOR_ARRAY_I',
  SensorArrayIi = 'SENSOR_ARRAY_II',
  SensorArrayIii = 'SENSOR_ARRAY_III',
  SurveyorI = 'SURVEYOR_I',
  SurveyorIi = 'SURVEYOR_II',
  SurveyorIii = 'SURVEYOR_III',
  TurretI = 'TURRET_I'
}

/**
 * ShipNavFlightMode : The ship's set speed when traveling between waypoints or systems.
 * The ship's set speed when traveling between waypoints or systems.
 */
export enum ShipNavFlightMode {
  Burn = 'BURN',
  Cruise = 'CRUISE',
  Drift = 'DRIFT',
  Stealth = 'STEALTH'
}

/**
 * ShipNavStatus : The current status of the ship
 * The current status of the ship
 */
export enum ShipNavStatus {
  Docked = 'DOCKED',
  InOrbit = 'IN_ORBIT',
  InTransit = 'IN_TRANSIT'
}

/** Symbol of the reactor. */
export enum ShipReactorSymbol {
  AntimatterI = 'ANTIMATTER_I',
  ChemicalI = 'CHEMICAL_I',
  FissionI = 'FISSION_I',
  FusionI = 'FUSION_I',
  SolarI = 'SOLAR_I'
}

/**
 * ShipRole : The registered role of the ship
 * The registered role of the ship
 */
export enum ShipRole {
  Carrier = 'CARRIER',
  Command = 'COMMAND',
  Excavator = 'EXCAVATOR',
  Explorer = 'EXPLORER',
  Fabricator = 'FABRICATOR',
  Harvester = 'HARVESTER',
  Hauler = 'HAULER',
  Interceptor = 'INTERCEPTOR',
  Patrol = 'PATROL',
  Refinery = 'REFINERY',
  Repair = 'REPAIR',
  Satellite = 'SATELLITE',
  Surveyor = 'SURVEYOR',
  Transport = 'TRANSPORT'
}

export type ShipState = {
  __typename?: 'ShipState';
  autoPilotArrival?: Maybe<Scalars['DateTime']['output']>;
  autoPilotDepartureTime?: Maybe<Scalars['DateTime']['output']>;
  autoPilotDestinationSymbol?: Maybe<Waypoint>;
  autoPilotDestinationSystemSymbol?: Maybe<System>;
  autoPilotDistance?: Maybe<Scalars['Float']['output']>;
  autoPilotFuelCost?: Maybe<Scalars['Int']['output']>;
  autoPilotOriginSymbol?: Maybe<Waypoint>;
  autoPilotOriginSystemSymbol?: Maybe<System>;
  autoPilotTravelTime?: Maybe<Scalars['Float']['output']>;
  cargoCapacity: Scalars['Int']['output'];
  cargoInventory: Scalars['JSONObject']['output'];
  cargoUnits: Scalars['Int']['output'];
  cooldown?: Maybe<Scalars['Int']['output']>;
  cooldownExpiration?: Maybe<Scalars['DateTime']['output']>;
  createdAt: Scalars['DateTime']['output'];
  displayName: Scalars['String']['output'];
  engineCondition: Scalars['Float']['output'];
  engineInfo: EngineInfo;
  engineIntegrity: Scalars['Float']['output'];
  engineSpeed: Scalars['Int']['output'];
  engineSymbol: ShipEngineSymbol;
  flightMode: Scalars['String']['output'];
  frameCondition: Scalars['Float']['output'];
  frameInfo: FrameInfo;
  frameIntegrity: Scalars['Float']['output'];
  frameSymbol: ShipFrameSymbol;
  fuelCapacity: Scalars['Int']['output'];
  fuelCurrent: Scalars['Int']['output'];
  id: Scalars['Int']['output'];
  moduleInfos: Array<ModuleInfo>;
  modules: Array<ShipModuleSymbol>;
  mountInfos: Array<MountInfo>;
  mounts: Array<ShipMountSymbol>;
  navStatus: Scalars['String']['output'];
  reactorCondition: Scalars['Float']['output'];
  reactorInfo: ReactorInfo;
  reactorIntegrity: Scalars['Float']['output'];
  reactorSymbol: ShipReactorSymbol;
  routeArrival: Scalars['DateTime']['output'];
  routeDeparture: Scalars['DateTime']['output'];
  routeDestinationSymbol?: Maybe<Waypoint>;
  routeDestinationSystem?: Maybe<System>;
  routeOriginSymbol?: Maybe<Waypoint>;
  routeOriginSystem?: Maybe<System>;
  ship?: Maybe<Ship>;
  symbol: Scalars['String']['output'];
  systemSymbol?: Maybe<System>;
  waypointSymbol?: Maybe<Waypoint>;
};

export type ShipStateBy =
  { shipSymbol: Scalars['String']['input']; system?: never; waypoint?: never; }
  |  { shipSymbol?: never; system: Scalars['String']['input']; waypoint?: never; }
  |  { shipSymbol?: never; system?: never; waypoint: Scalars['String']['input']; };

export type ShipStatePage = {
  __typename?: 'ShipStatePage';
  items: Array<ShipState>;
  page: Scalars['Int']['output'];
  pageSize?: Maybe<Scalars['Int']['output']>;
  totalCount: Scalars['Int']['output'];
};

export type ShipStatus = {
  __typename?: 'ShipStatus';
  assignment?: Maybe<ShipAssignment>;
  assignmentId?: Maybe<Scalars['Int']['output']>;
  fleet?: Maybe<Fleet>;
  fleetId?: Maybe<Scalars['Int']['output']>;
  status: AssignmentStatus;
  tempAssignment?: Maybe<ShipAssignment>;
  tempAssignmentId?: Maybe<Scalars['Int']['output']>;
  tempFleet?: Maybe<Fleet>;
  tempFleetId?: Maybe<Scalars['Int']['output']>;
  waitingForApi: Scalars['Boolean']['output'];
  waitingForManager: Scalars['Boolean']['output'];
};

/**
 * ShipType : Type of ship
 * Type of ship
 */
export enum ShipType {
  BulkFreighter = 'BULK_FREIGHTER',
  CommandFrigate = 'COMMAND_FRIGATE',
  Explorer = 'EXPLORER',
  HeavyFreighter = 'HEAVY_FREIGHTER',
  Interceptor = 'INTERCEPTOR',
  LightHauler = 'LIGHT_HAULER',
  LightShuttle = 'LIGHT_SHUTTLE',
  MiningDrone = 'MINING_DRONE',
  OreHound = 'ORE_HOUND',
  Probe = 'PROBE',
  RefiningFreighter = 'REFINING_FREIGHTER',
  SiphonDrone = 'SIPHON_DRONE',
  Surveyor = 'SURVEYOR'
}

export enum ShipmentStatus {
  Delivered = 'DELIVERED',
  Failed = 'FAILED',
  InTransit = 'IN_TRANSIT'
}

export enum ShippingStatus {
  Delivering = 'DELIVERING',
  InTransitToDelivery = 'IN_TRANSIT_TO_DELIVERY',
  InTransitToPurchase = 'IN_TRANSIT_TO_PURCHASE',
  Purchasing = 'PURCHASING',
  Unknown = 'UNKNOWN'
}

export type Shipyard = {
  __typename?: 'Shipyard';
  createdAt: Scalars['DateTime']['output'];
  history: ShipyardPage;
  id: Scalars['Int']['output'];
  modificationsFee: Scalars['Int']['output'];
  shipyardShipTypes: ShipyardShipTypesPage;
  shipyardShips: ShipyardShipPage;
  shipyardTransactions: ShipyardTransactionPage;
  waypoint?: Maybe<Waypoint>;
  waypointSymbol: Scalars['String']['output'];
};


export type ShipyardHistoryArgs = {
  page?: InputMaybe<Scalars['Int']['input']>;
  pageSize?: InputMaybe<Scalars['Int']['input']>;
};


export type ShipyardShipyardShipTypesArgs = {
  page?: InputMaybe<Scalars['Int']['input']>;
  pageSize?: InputMaybe<Scalars['Int']['input']>;
};


export type ShipyardShipyardShipsArgs = {
  page?: InputMaybe<Scalars['Int']['input']>;
  pageSize?: InputMaybe<Scalars['Int']['input']>;
};


export type ShipyardShipyardTransactionsArgs = {
  page?: InputMaybe<Scalars['Int']['input']>;
  pageSize?: InputMaybe<Scalars['Int']['input']>;
};

export type ShipyardPage = {
  __typename?: 'ShipyardPage';
  items: Array<Shipyard>;
  page: Scalars['Int']['output'];
  pageSize?: Maybe<Scalars['Int']['output']>;
  totalCount: Scalars['Int']['output'];
};

export type ShipyardShip = {
  __typename?: 'ShipyardShip';
  activity?: Maybe<ActivityLevel>;
  createdAt: Scalars['DateTime']['output'];
  crewCapacity: Scalars['Int']['output'];
  crewRequirement: Scalars['Int']['output'];
  engineQuality?: Maybe<Scalars['Float']['output']>;
  engineType: ShipEngineSymbol;
  frameQuality?: Maybe<Scalars['Float']['output']>;
  frameType: ShipFrameSymbol;
  history: ShipyardShipPage;
  id: Scalars['Int']['output'];
  modules: Array<ShipModuleSymbol>;
  mounts: Array<ShipMountSymbol>;
  name: Scalars['String']['output'];
  purchasePrice: Scalars['Int']['output'];
  reactorQuality?: Maybe<Scalars['Float']['output']>;
  reactorType: ShipReactorSymbol;
  shipType: ShipType;
  shipyardTransactions: ShipyardTransactionPage;
  supply: SupplyLevel;
  tradeSymbolInfo: TradeSymbolInfo;
  waypoint?: Maybe<Waypoint>;
  waypointSymbol: Scalars['String']['output'];
};


export type ShipyardShipHistoryArgs = {
  page?: InputMaybe<Scalars['Int']['input']>;
  pageSize?: InputMaybe<Scalars['Int']['input']>;
};


export type ShipyardShipShipyardTransactionsArgs = {
  page?: InputMaybe<Scalars['Int']['input']>;
  pageSize?: InputMaybe<Scalars['Int']['input']>;
};

export type ShipyardShipBy =
  { shipSymbol: ShipType; system?: never; waypoint?: never; }
  |  { shipSymbol?: never; system: Scalars['String']['input']; waypoint?: never; }
  |  { shipSymbol?: never; system?: never; waypoint: Scalars['String']['input']; };

export type ShipyardShipPage = {
  __typename?: 'ShipyardShipPage';
  items: Array<ShipyardShip>;
  page: Scalars['Int']['output'];
  pageSize?: Maybe<Scalars['Int']['output']>;
  totalCount: Scalars['Int']['output'];
};

export type ShipyardShipTypes = {
  __typename?: 'ShipyardShipTypes';
  createdAt: Scalars['DateTime']['output'];
  id: Scalars['Int']['output'];
  shipType: ShipType;
  shipyard?: Maybe<Shipyard>;
  shipyardId: Scalars['Int']['output'];
  tradeSymbolInfo: TradeSymbolInfo;
};

export type ShipyardShipTypesPage = {
  __typename?: 'ShipyardShipTypesPage';
  items: Array<ShipyardShipTypes>;
  page: Scalars['Int']['output'];
  pageSize?: Maybe<Scalars['Int']['output']>;
  totalCount: Scalars['Int']['output'];
};

export type ShipyardTransaction = {
  __typename?: 'ShipyardTransaction';
  agent?: Maybe<Agent>;
  agentSymbol: Scalars['String']['output'];
  id: Scalars['Int']['output'];
  price: Scalars['Int']['output'];
  ship?: Maybe<Ship>;
  shipType: ShipType;
  shipyardShip?: Maybe<ShipyardShip>;
  timestamp: Scalars['DateTime']['output'];
  tradeSymbolInfo: TradeSymbolInfo;
  waypoint?: Maybe<Waypoint>;
  waypointSymbol: Scalars['String']['output'];
};

export type ShipyardTransactionBy =
  { agent: Scalars['String']['input']; system?: never; type?: never; waypoint?: never; }
  |  { agent?: never; system: Scalars['String']['input']; type?: never; waypoint?: never; }
  |  { agent?: never; system?: never; type: ShipType; waypoint?: never; }
  |  { agent?: never; system?: never; type?: never; waypoint: Scalars['String']['input']; };

export type ShipyardTransactionPage = {
  __typename?: 'ShipyardTransactionPage';
  items: Array<ShipyardTransaction>;
  page: Scalars['Int']['output'];
  pageSize?: Maybe<Scalars['Int']['output']>;
  totalCount: Scalars['Int']['output'];
};

export type SiphonerAssignment = {
  __typename?: 'SiphonerAssignment';
  extractions?: Maybe<Scalars['Int']['output']>;
  state: ExtractorState;
  waypointSymbol?: Maybe<Scalars['String']['output']>;
};

/**
 * SupplyLevel : The supply level of a trade good.
 * The supply level of a trade good.
 */
export enum SupplyLevel {
  Abundant = 'ABUNDANT',
  High = 'HIGH',
  Limited = 'LIMITED',
  Moderate = 'MODERATE',
  Scarce = 'SCARCE'
}

export type Survey = {
  __typename?: 'Survey';
  createdAt: Scalars['DateTime']['output'];
  deposits: Array<TradeSymbol>;
  exhaustedSince?: Maybe<Scalars['DateTime']['output']>;
  expiration: Scalars['DateTime']['output'];
  extractions: ExtractionPage;
  percent: Array<SurveyPercent>;
  ship?: Maybe<Ship>;
  shipInfoAfter: Scalars['Int']['output'];
  shipInfoBefore: Scalars['Int']['output'];
  shipStateAfter?: Maybe<ShipState>;
  shipStateBefore?: Maybe<ShipState>;
  shipSymbol: Scalars['String']['output'];
  signature: Scalars['String']['output'];
  size: SurveySize;
  updatedAt: Scalars['DateTime']['output'];
  waypoint?: Maybe<Waypoint>;
  waypointSymbol: Scalars['String']['output'];
};


export type SurveyExtractionsArgs = {
  page?: InputMaybe<Scalars['Int']['input']>;
  pageSize?: InputMaybe<Scalars['Int']['input']>;
};

export type SurveyBy =
  { shipSymbol: Scalars['String']['input']; size?: never; system?: never; waypoint?: never; }
  |  { shipSymbol?: never; size: SurveySize; system?: never; waypoint?: never; }
  |  { shipSymbol?: never; size?: never; system: Scalars['String']['input']; waypoint?: never; }
  |  { shipSymbol?: never; size?: never; system?: never; waypoint: Scalars['String']['input']; };

export type SurveyPage = {
  __typename?: 'SurveyPage';
  items: Array<Survey>;
  page: Scalars['Int']['output'];
  pageSize?: Maybe<Scalars['Int']['output']>;
  totalCount: Scalars['Int']['output'];
};

export type SurveyPercent = {
  __typename?: 'SurveyPercent';
  percent: Scalars['Float']['output'];
  symbol: TradeSymbol;
  tradeSymbolInfo: TradeSymbolInfo;
};

/**
 * SurveySize : The size of the deposit. This value indicates how much can be extracted from the survey before it is exhausted.
 * The size of the deposit. This value indicates how much can be extracted from the survey before it is exhausted.
 */
export enum SurveySize {
  Large = 'LARGE',
  Moderate = 'MODERATE',
  Small = 'SMALL'
}

export type SurveyorAssignment = {
  __typename?: 'SurveyorAssignment';
  surveys?: Maybe<Scalars['Int']['output']>;
  waypointSymbol?: Maybe<Scalars['String']['output']>;
};

export type System = {
  __typename?: 'System';
  chartTransactions: ChartTransactionPage;
  constellation?: Maybe<Scalars['String']['output']>;
  constructionMaterials: ConstructionMaterialPage;
  constructionShipments: ConstructionShipmentPage;
  contractDeliveries: ContractDeliveryPage;
  extractions: ExtractionPage;
  fleets: FleetPage;
  jumpGateConnections: JumpGateConnectionPage;
  marketTradeGoods: MarketTradeGoodPage;
  marketTrades: MarketTradePage;
  marketTransactions: MarketTransactionPage;
  populationDisabled: Scalars['Boolean']['output'];
  repairTransactions: RepairTransactionPage;
  scrapTransactions: ScrapTransactionPage;
  sectorSymbol: Scalars['String']['output'];
  seenAgents: Array<KnownAgent>;
  shipModificationTransactions: ShipModificationTransactionPage;
  ships: Array<Ship>;
  shipyardShipTypes: ShipyardShipTypesPage;
  shipyardShips: ShipyardShipPage;
  shipyardTransactions: ShipyardTransactionPage;
  surveys: SurveyPage;
  symbol: Scalars['String']['output'];
  systemType: SystemType;
  tradeRoutes: TradeRoutePage;
  waypoints: WaypointPage;
  x: Scalars['Int']['output'];
  y: Scalars['Int']['output'];
};


export type SystemChartTransactionsArgs = {
  page?: InputMaybe<Scalars['Int']['input']>;
  pageSize?: InputMaybe<Scalars['Int']['input']>;
};


export type SystemConstructionMaterialsArgs = {
  page?: InputMaybe<Scalars['Int']['input']>;
  pageSize?: InputMaybe<Scalars['Int']['input']>;
};


export type SystemConstructionShipmentsArgs = {
  page?: InputMaybe<Scalars['Int']['input']>;
  pageSize?: InputMaybe<Scalars['Int']['input']>;
};


export type SystemContractDeliveriesArgs = {
  page?: InputMaybe<Scalars['Int']['input']>;
  pageSize?: InputMaybe<Scalars['Int']['input']>;
};


export type SystemExtractionsArgs = {
  page?: InputMaybe<Scalars['Int']['input']>;
  pageSize?: InputMaybe<Scalars['Int']['input']>;
};


export type SystemFleetsArgs = {
  page?: InputMaybe<Scalars['Int']['input']>;
  pageSize?: InputMaybe<Scalars['Int']['input']>;
};


export type SystemJumpGateConnectionsArgs = {
  page?: InputMaybe<Scalars['Int']['input']>;
  pageSize?: InputMaybe<Scalars['Int']['input']>;
};


export type SystemMarketTradeGoodsArgs = {
  page?: InputMaybe<Scalars['Int']['input']>;
  pageSize?: InputMaybe<Scalars['Int']['input']>;
};


export type SystemMarketTradesArgs = {
  page?: InputMaybe<Scalars['Int']['input']>;
  pageSize?: InputMaybe<Scalars['Int']['input']>;
};


export type SystemMarketTransactionsArgs = {
  page?: InputMaybe<Scalars['Int']['input']>;
  pageSize?: InputMaybe<Scalars['Int']['input']>;
};


export type SystemRepairTransactionsArgs = {
  page?: InputMaybe<Scalars['Int']['input']>;
  pageSize?: InputMaybe<Scalars['Int']['input']>;
};


export type SystemScrapTransactionsArgs = {
  page?: InputMaybe<Scalars['Int']['input']>;
  pageSize?: InputMaybe<Scalars['Int']['input']>;
};


export type SystemShipModificationTransactionsArgs = {
  page?: InputMaybe<Scalars['Int']['input']>;
  pageSize?: InputMaybe<Scalars['Int']['input']>;
};


export type SystemShipyardShipTypesArgs = {
  page?: InputMaybe<Scalars['Int']['input']>;
  pageSize?: InputMaybe<Scalars['Int']['input']>;
};


export type SystemShipyardShipsArgs = {
  page?: InputMaybe<Scalars['Int']['input']>;
  pageSize?: InputMaybe<Scalars['Int']['input']>;
};


export type SystemShipyardTransactionsArgs = {
  page?: InputMaybe<Scalars['Int']['input']>;
  pageSize?: InputMaybe<Scalars['Int']['input']>;
};


export type SystemSurveysArgs = {
  page?: InputMaybe<Scalars['Int']['input']>;
  pageSize?: InputMaybe<Scalars['Int']['input']>;
};


export type SystemTradeRoutesArgs = {
  page?: InputMaybe<Scalars['Int']['input']>;
  pageSize?: InputMaybe<Scalars['Int']['input']>;
};


export type SystemWaypointsArgs = {
  page?: InputMaybe<Scalars['Int']['input']>;
  pageSize?: InputMaybe<Scalars['Int']['input']>;
};

export type SystemPage = {
  __typename?: 'SystemPage';
  items: Array<System>;
  page: Scalars['Int']['output'];
  pageSize?: Maybe<Scalars['Int']['output']>;
  totalCount: Scalars['Int']['output'];
};

/**
 * SystemType : The type of system.
 * The type of system.
 */
export enum SystemType {
  BlackHole = 'BLACK_HOLE',
  BlueStar = 'BLUE_STAR',
  Hypergiant = 'HYPERGIANT',
  Nebula = 'NEBULA',
  NeutronStar = 'NEUTRON_STAR',
  OrangeStar = 'ORANGE_STAR',
  RedStar = 'RED_STAR',
  Unstable = 'UNSTABLE',
  WhiteDwarf = 'WHITE_DWARF',
  YoungStar = 'YOUNG_STAR'
}

export type TradeManagerInfo = {
  __typename?: 'TradeManagerInfo';
  busy: Scalars['Boolean']['output'];
  channelState: ChannelInfo;
};

export enum TradeMode {
  ProfitPerApiRequest = 'PROFIT_PER_API_REQUEST',
  ProfitPerHour = 'PROFIT_PER_HOUR',
  ProfitPerTrip = 'PROFIT_PER_TRIP'
}

export type TradeRoute = {
  __typename?: 'TradeRoute';
  PurchaseWaypointSymbol: Scalars['String']['output'];
  SellWaypointSymbol: Scalars['String']['output'];
  createdAt: Scalars['DateTime']['output'];
  id: Scalars['Int']['output'];
  marketTransactionSummary: TransactionSummary;
  predictedPurchasePrice: Scalars['Int']['output'];
  predictedSellPrice: Scalars['Int']['output'];
  purchaseMarketTradeGood?: Maybe<MarketTradeGood>;
  purchaseWaypoint?: Maybe<Waypoint>;
  reservation?: Maybe<ReservedFund>;
  reservedFund?: Maybe<Scalars['Int']['output']>;
  sellMarketTradeGood?: Maybe<MarketTradeGood>;
  sellWaypoint?: Maybe<Waypoint>;
  ship?: Maybe<Ship>;
  shipSymbol: Scalars['String']['output'];
  status: ShipmentStatus;
  symbol: TradeSymbol;
  tradeSymbolInfo: TradeSymbolInfo;
  tradeVolume: Scalars['Int']['output'];
  transactions: MarketTransactionPage;
};


export type TradeRouteTransactionsArgs = {
  page?: InputMaybe<Scalars['Int']['input']>;
  pageSize?: InputMaybe<Scalars['Int']['input']>;
};

export type TradeRoutePage = {
  __typename?: 'TradeRoutePage';
  items: Array<TradeRoute>;
  page: Scalars['Int']['output'];
  pageSize?: Maybe<Scalars['Int']['output']>;
  totalCount: Scalars['Int']['output'];
};

/**
 * TradeSymbol : The good's symbol.
 * The good's symbol.
 */
export enum TradeSymbol {
  AdvancedCircuitry = 'ADVANCED_CIRCUITRY',
  AiMainframes = 'AI_MAINFRAMES',
  Aluminum = 'ALUMINUM',
  AluminumOre = 'ALUMINUM_ORE',
  AmmoniaIce = 'AMMONIA_ICE',
  Ammunition = 'AMMUNITION',
  Antimatter = 'ANTIMATTER',
  AssaultRifles = 'ASSAULT_RIFLES',
  Biocomposites = 'BIOCOMPOSITES',
  BotanicalSpecimens = 'BOTANICAL_SPECIMENS',
  Clothing = 'CLOTHING',
  Copper = 'COPPER',
  CopperOre = 'COPPER_ORE',
  CulturalArtifacts = 'CULTURAL_ARTIFACTS',
  CyberImplants = 'CYBER_IMPLANTS',
  Diamonds = 'DIAMONDS',
  Drugs = 'DRUGS',
  Electronics = 'ELECTRONICS',
  EngineHyperDriveI = 'ENGINE_HYPER_DRIVE_I',
  EngineImpulseDriveI = 'ENGINE_IMPULSE_DRIVE_I',
  EngineIonDriveI = 'ENGINE_ION_DRIVE_I',
  EngineIonDriveIi = 'ENGINE_ION_DRIVE_II',
  Equipment = 'EQUIPMENT',
  ExoticMatter = 'EXOTIC_MATTER',
  Explosives = 'EXPLOSIVES',
  Fabrics = 'FABRICS',
  FabMats = 'FAB_MATS',
  Fertilizers = 'FERTILIZERS',
  Firearms = 'FIREARMS',
  Food = 'FOOD',
  FrameBulkFreighter = 'FRAME_BULK_FREIGHTER',
  FrameCarrier = 'FRAME_CARRIER',
  FrameCruiser = 'FRAME_CRUISER',
  FrameDestroyer = 'FRAME_DESTROYER',
  FrameDrone = 'FRAME_DRONE',
  FrameExplorer = 'FRAME_EXPLORER',
  FrameFighter = 'FRAME_FIGHTER',
  FrameFrigate = 'FRAME_FRIGATE',
  FrameHeavyFreighter = 'FRAME_HEAVY_FREIGHTER',
  FrameInterceptor = 'FRAME_INTERCEPTOR',
  FrameLightFreighter = 'FRAME_LIGHT_FREIGHTER',
  FrameMiner = 'FRAME_MINER',
  FrameProbe = 'FRAME_PROBE',
  FrameRacer = 'FRAME_RACER',
  FrameShuttle = 'FRAME_SHUTTLE',
  FrameTransport = 'FRAME_TRANSPORT',
  Fuel = 'FUEL',
  GeneTherapeutics = 'GENE_THERAPEUTICS',
  Gold = 'GOLD',
  GoldOre = 'GOLD_ORE',
  GravitonEmitters = 'GRAVITON_EMITTERS',
  Holographics = 'HOLOGRAPHICS',
  Hydrocarbon = 'HYDROCARBON',
  IceWater = 'ICE_WATER',
  Iron = 'IRON',
  IronOre = 'IRON_ORE',
  Jewelry = 'JEWELRY',
  LabInstruments = 'LAB_INSTRUMENTS',
  LaserRifles = 'LASER_RIFLES',
  LiquidHydrogen = 'LIQUID_HYDROGEN',
  LiquidNitrogen = 'LIQUID_NITROGEN',
  Machinery = 'MACHINERY',
  Medicine = 'MEDICINE',
  Meritium = 'MERITIUM',
  MeritiumOre = 'MERITIUM_ORE',
  Microprocessors = 'MICROPROCESSORS',
  MicroFusionGenerators = 'MICRO_FUSION_GENERATORS',
  MilitaryEquipment = 'MILITARY_EQUIPMENT',
  ModuleCargoHoldI = 'MODULE_CARGO_HOLD_I',
  ModuleCargoHoldIi = 'MODULE_CARGO_HOLD_II',
  ModuleCargoHoldIii = 'MODULE_CARGO_HOLD_III',
  ModuleCrewQuartersI = 'MODULE_CREW_QUARTERS_I',
  ModuleEnvoyQuartersI = 'MODULE_ENVOY_QUARTERS_I',
  ModuleFuelRefineryI = 'MODULE_FUEL_REFINERY_I',
  ModuleGasProcessorI = 'MODULE_GAS_PROCESSOR_I',
  ModuleJumpDriveI = 'MODULE_JUMP_DRIVE_I',
  ModuleJumpDriveIi = 'MODULE_JUMP_DRIVE_II',
  ModuleJumpDriveIii = 'MODULE_JUMP_DRIVE_III',
  ModuleMicroRefineryI = 'MODULE_MICRO_REFINERY_I',
  ModuleMineralProcessorI = 'MODULE_MINERAL_PROCESSOR_I',
  ModuleOreRefineryI = 'MODULE_ORE_REFINERY_I',
  ModulePassengerCabinI = 'MODULE_PASSENGER_CABIN_I',
  ModuleScienceLabI = 'MODULE_SCIENCE_LAB_I',
  ModuleShieldGeneratorI = 'MODULE_SHIELD_GENERATOR_I',
  ModuleShieldGeneratorIi = 'MODULE_SHIELD_GENERATOR_II',
  ModuleWarpDriveI = 'MODULE_WARP_DRIVE_I',
  ModuleWarpDriveIi = 'MODULE_WARP_DRIVE_II',
  ModuleWarpDriveIii = 'MODULE_WARP_DRIVE_III',
  MoodRegulators = 'MOOD_REGULATORS',
  MountGasSiphonI = 'MOUNT_GAS_SIPHON_I',
  MountGasSiphonIi = 'MOUNT_GAS_SIPHON_II',
  MountGasSiphonIii = 'MOUNT_GAS_SIPHON_III',
  MountLaserCannonI = 'MOUNT_LASER_CANNON_I',
  MountMiningLaserI = 'MOUNT_MINING_LASER_I',
  MountMiningLaserIi = 'MOUNT_MINING_LASER_II',
  MountMiningLaserIii = 'MOUNT_MINING_LASER_III',
  MountMissileLauncherI = 'MOUNT_MISSILE_LAUNCHER_I',
  MountSensorArrayI = 'MOUNT_SENSOR_ARRAY_I',
  MountSensorArrayIi = 'MOUNT_SENSOR_ARRAY_II',
  MountSensorArrayIii = 'MOUNT_SENSOR_ARRAY_III',
  MountSurveyorI = 'MOUNT_SURVEYOR_I',
  MountSurveyorIi = 'MOUNT_SURVEYOR_II',
  MountSurveyorIii = 'MOUNT_SURVEYOR_III',
  MountTurretI = 'MOUNT_TURRET_I',
  Nanobots = 'NANOBOTS',
  NeuralChips = 'NEURAL_CHIPS',
  NovelLifeforms = 'NOVEL_LIFEFORMS',
  Plastics = 'PLASTICS',
  Platinum = 'PLATINUM',
  PlatinumOre = 'PLATINUM_ORE',
  Polynucleotides = 'POLYNUCLEOTIDES',
  PreciousStones = 'PRECIOUS_STONES',
  QuantumDrives = 'QUANTUM_DRIVES',
  QuantumStabilizers = 'QUANTUM_STABILIZERS',
  QuartzSand = 'QUARTZ_SAND',
  ReactorAntimatterI = 'REACTOR_ANTIMATTER_I',
  ReactorChemicalI = 'REACTOR_CHEMICAL_I',
  ReactorFissionI = 'REACTOR_FISSION_I',
  ReactorFusionI = 'REACTOR_FUSION_I',
  ReactorSolarI = 'REACTOR_SOLAR_I',
  RelicTech = 'RELIC_TECH',
  RoboticDrones = 'ROBOTIC_DRONES',
  ShipBulkFreighter = 'SHIP_BULK_FREIGHTER',
  ShipCommandFrigate = 'SHIP_COMMAND_FRIGATE',
  ShipExplorer = 'SHIP_EXPLORER',
  ShipHeavyFreighter = 'SHIP_HEAVY_FREIGHTER',
  ShipInterceptor = 'SHIP_INTERCEPTOR',
  ShipLightHauler = 'SHIP_LIGHT_HAULER',
  ShipLightShuttle = 'SHIP_LIGHT_SHUTTLE',
  ShipMiningDrone = 'SHIP_MINING_DRONE',
  ShipOreHound = 'SHIP_ORE_HOUND',
  ShipParts = 'SHIP_PARTS',
  ShipPlating = 'SHIP_PLATING',
  ShipProbe = 'SHIP_PROBE',
  ShipRefiningFreighter = 'SHIP_REFINING_FREIGHTER',
  ShipSalvage = 'SHIP_SALVAGE',
  ShipSiphonDrone = 'SHIP_SIPHON_DRONE',
  ShipSurveyor = 'SHIP_SURVEYOR',
  SiliconCrystals = 'SILICON_CRYSTALS',
  Silver = 'SILVER',
  SilverOre = 'SILVER_ORE',
  Supergrains = 'SUPERGRAINS',
  Uranite = 'URANITE',
  UraniteOre = 'URANITE_ORE',
  ViralAgents = 'VIRAL_AGENTS'
}

export type TradeSymbolInfo = {
  __typename?: 'TradeSymbolInfo';
  requiredBy: TradeSymbolInfoPage;
  requires: TradeSymbolInfoPage;
  symbol: TradeSymbol;
};


export type TradeSymbolInfoRequiredByArgs = {
  page?: InputMaybe<Scalars['Int']['input']>;
  pageSize?: InputMaybe<Scalars['Int']['input']>;
};


export type TradeSymbolInfoRequiresArgs = {
  page?: InputMaybe<Scalars['Int']['input']>;
  pageSize?: InputMaybe<Scalars['Int']['input']>;
};

export type TradeSymbolInfoPage = {
  __typename?: 'TradeSymbolInfoPage';
  items: Array<TradeSymbolInfo>;
  page: Scalars['Int']['output'];
  pageSize?: Maybe<Scalars['Int']['output']>;
  totalCount: Scalars['Int']['output'];
};

export type TraderStatus = {
  __typename?: 'TraderStatus';
  cycle?: Maybe<Scalars['Int']['output']>;
  onSleep: Scalars['Boolean']['output'];
  shipmentId?: Maybe<Scalars['Int']['output']>;
  shippingStatus?: Maybe<ShippingStatus>;
  tradeRoute?: Maybe<TradeRoute>;
  waitingForManager: Scalars['Boolean']['output'];
};

export type TradingConfig = {
  __typename?: 'TradingConfig';
  marketBlacklist: Array<TradeSymbol>;
  marketPreferList: Array<TradeSymbol>;
  minCargoSpace: Scalars['Int']['output'];
  purchaseMultiplier: Scalars['Float']['output'];
  shipMarketRatio: Scalars['Float']['output'];
  tradeMode: TradeMode;
  tradeProfitThreshold: Scalars['Int']['output'];
};

export type TransactionSummary = {
  __typename?: 'TransactionSummary';
  expenses?: Maybe<Scalars['Int']['output']>;
  income?: Maybe<Scalars['Int']['output']>;
  purchaseTransactions?: Maybe<Scalars['Int']['output']>;
  purchaseUnits?: Maybe<Scalars['Int']['output']>;
  sellTransactions?: Maybe<Scalars['Int']['output']>;
  sellUnits?: Maybe<Scalars['Int']['output']>;
  sum?: Maybe<Scalars['Int']['output']>;
  transactions?: Maybe<Scalars['Int']['output']>;
  units?: Maybe<Scalars['Int']['output']>;
};

export type TransferStatus = {
  __typename?: 'TransferStatus';
  assignmentId: Scalars['Int']['output'];
  fleetId: Scalars['Int']['output'];
  systemSymbol: Scalars['String']['output'];
};

export type TransporterAssignment = {
  __typename?: 'TransporterAssignment';
  cycles?: Maybe<Scalars['Int']['output']>;
  state: TransporterState;
  waypointSymbol?: Maybe<Scalars['String']['output']>;
};

export enum TransporterState {
  InTransitToAsteroid = 'IN_TRANSIT_TO_ASTEROID',
  InTransitToMarket = 'IN_TRANSIT_TO_MARKET',
  LoadingCargo = 'LOADING_CARGO',
  SellingCargo = 'SELLING_CARGO',
  Unknown = 'UNKNOWN',
  WaitingForCargo = 'WAITING_FOR_CARGO'
}

/** The type of trade good (export, import, or exchange). */
export enum Type {
  Exchange = 'EXCHANGE',
  Export = 'EXPORT',
  Import = 'IMPORT'
}

export type UselessAssignment = {
  __typename?: 'UselessAssignment';
  controlled: Scalars['Boolean']['output'];
};

export type WarpConnection = {
  __typename?: 'WarpConnection';
  distance: Scalars['Float']['output'];
  end?: Maybe<Waypoint>;
  endIsMarketplace: Scalars['Boolean']['output'];
  endSymbol: Scalars['String']['output'];
  endSystem?: Maybe<System>;
  navMode: ShipNavFlightMode;
  refuel: Refuel;
  start?: Maybe<Waypoint>;
  startIsMarketplace: Scalars['Boolean']['output'];
  startSymbol: Scalars['String']['output'];
  startSystem?: Maybe<System>;
  travelTime: Scalars['Float']['output'];
};

export type Waypoint = {
  __typename?: 'Waypoint';
  chartTransaction?: Maybe<ChartTransaction>;
  chartedBy?: Maybe<Scalars['String']['output']>;
  chartedOn?: Maybe<Scalars['String']['output']>;
  constructionMaterials: ConstructionMaterialPage;
  constructionShipmentsFrom: ConstructionShipmentPage;
  constructionShipmentsTo: ConstructionShipmentPage;
  contractDeliveries: ContractDeliveryPage;
  contractShipmentsFrom: ContractShipmentPage;
  contractShipmentsTo: ContractShipmentPage;
  createdAt: Scalars['DateTime']['output'];
  extractions: ExtractionPage;
  faction?: Maybe<Scalars['String']['output']>;
  hasMarketplace: Scalars['Boolean']['output'];
  hasShipyard: Scalars['Boolean']['output'];
  isUnderConstruction: Scalars['Boolean']['output'];
  jumpGateConnections: JumpGateConnectionPage;
  lastScrap?: Maybe<Scalars['DateTime']['output']>;
  marketTradeGoods: MarketTradeGoodPage;
  marketTrades: MarketTradePage;
  marketTransactionSummary: TransactionSummary;
  marketTransactions: MarketTransactionPage;
  modifiers: Array<WaypointModifierSymbol>;
  nextScrap?: Maybe<Scalars['DateTime']['output']>;
  orbitals: Array<Scalars['String']['output']>;
  orbits?: Maybe<Scalars['String']['output']>;
  repairTransactions: RepairTransactionPage;
  scrapTransactions: ScrapTransactionPage;
  shipModificationTransactions: ShipModificationTransactionPage;
  ships: Array<Ship>;
  shipyard?: Maybe<Shipyard>;
  shipyardShipTypes: ShipyardShipTypesPage;
  shipyardShips: ShipyardShipPage;
  shipyardTransactions: ShipyardTransactionPage;
  surveys: SurveyPage;
  symbol: Scalars['String']['output'];
  system?: Maybe<System>;
  systemSymbol: Scalars['String']['output'];
  tradeRoutes: TradeRoutePage;
  tradeRoutesFrom: TradeRoutePage;
  tradeRoutesTo: TradeRoutePage;
  traits: Array<WaypointTraitSymbol>;
  unstableSince?: Maybe<Scalars['DateTime']['output']>;
  waypointType: WaypointType;
  x: Scalars['Int']['output'];
  y: Scalars['Int']['output'];
};


export type WaypointConstructionMaterialsArgs = {
  page?: InputMaybe<Scalars['Int']['input']>;
  pageSize?: InputMaybe<Scalars['Int']['input']>;
};


export type WaypointConstructionShipmentsFromArgs = {
  page?: InputMaybe<Scalars['Int']['input']>;
  pageSize?: InputMaybe<Scalars['Int']['input']>;
};


export type WaypointConstructionShipmentsToArgs = {
  page?: InputMaybe<Scalars['Int']['input']>;
  pageSize?: InputMaybe<Scalars['Int']['input']>;
};


export type WaypointContractDeliveriesArgs = {
  page?: InputMaybe<Scalars['Int']['input']>;
  pageSize?: InputMaybe<Scalars['Int']['input']>;
};


export type WaypointContractShipmentsFromArgs = {
  page?: InputMaybe<Scalars['Int']['input']>;
  pageSize?: InputMaybe<Scalars['Int']['input']>;
};


export type WaypointContractShipmentsToArgs = {
  page?: InputMaybe<Scalars['Int']['input']>;
  pageSize?: InputMaybe<Scalars['Int']['input']>;
};


export type WaypointExtractionsArgs = {
  page?: InputMaybe<Scalars['Int']['input']>;
  pageSize?: InputMaybe<Scalars['Int']['input']>;
};


export type WaypointJumpGateConnectionsArgs = {
  page?: InputMaybe<Scalars['Int']['input']>;
  pageSize?: InputMaybe<Scalars['Int']['input']>;
};


export type WaypointMarketTradeGoodsArgs = {
  page?: InputMaybe<Scalars['Int']['input']>;
  pageSize?: InputMaybe<Scalars['Int']['input']>;
};


export type WaypointMarketTradesArgs = {
  page?: InputMaybe<Scalars['Int']['input']>;
  pageSize?: InputMaybe<Scalars['Int']['input']>;
};


export type WaypointMarketTransactionsArgs = {
  page?: InputMaybe<Scalars['Int']['input']>;
  pageSize?: InputMaybe<Scalars['Int']['input']>;
};


export type WaypointRepairTransactionsArgs = {
  page?: InputMaybe<Scalars['Int']['input']>;
  pageSize?: InputMaybe<Scalars['Int']['input']>;
};


export type WaypointScrapTransactionsArgs = {
  page?: InputMaybe<Scalars['Int']['input']>;
  pageSize?: InputMaybe<Scalars['Int']['input']>;
};


export type WaypointShipModificationTransactionsArgs = {
  page?: InputMaybe<Scalars['Int']['input']>;
  pageSize?: InputMaybe<Scalars['Int']['input']>;
};


export type WaypointShipyardShipTypesArgs = {
  page?: InputMaybe<Scalars['Int']['input']>;
  pageSize?: InputMaybe<Scalars['Int']['input']>;
};


export type WaypointShipyardShipsArgs = {
  page?: InputMaybe<Scalars['Int']['input']>;
  pageSize?: InputMaybe<Scalars['Int']['input']>;
};


export type WaypointShipyardTransactionsArgs = {
  page?: InputMaybe<Scalars['Int']['input']>;
  pageSize?: InputMaybe<Scalars['Int']['input']>;
};


export type WaypointSurveysArgs = {
  page?: InputMaybe<Scalars['Int']['input']>;
  pageSize?: InputMaybe<Scalars['Int']['input']>;
};


export type WaypointTradeRoutesArgs = {
  page?: InputMaybe<Scalars['Int']['input']>;
  pageSize?: InputMaybe<Scalars['Int']['input']>;
};


export type WaypointTradeRoutesFromArgs = {
  page?: InputMaybe<Scalars['Int']['input']>;
  pageSize?: InputMaybe<Scalars['Int']['input']>;
};


export type WaypointTradeRoutesToArgs = {
  page?: InputMaybe<Scalars['Int']['input']>;
  pageSize?: InputMaybe<Scalars['Int']['input']>;
};

/**
 * WaypointModifierSymbol : The unique identifier of the modifier.
 * The unique identifier of the modifier.
 */
export enum WaypointModifierSymbol {
  CivilUnrest = 'CIVIL_UNREST',
  CriticalLimit = 'CRITICAL_LIMIT',
  RadiationLeak = 'RADIATION_LEAK',
  Stripped = 'STRIPPED',
  Unstable = 'UNSTABLE'
}

export type WaypointPage = {
  __typename?: 'WaypointPage';
  items: Array<Waypoint>;
  page: Scalars['Int']['output'];
  pageSize?: Maybe<Scalars['Int']['output']>;
  totalCount: Scalars['Int']['output'];
};

/**
 * WaypointTraitSymbol : The unique identifier of the trait.
 * The unique identifier of the trait.
 */
export enum WaypointTraitSymbol {
  AshClouds = 'ASH_CLOUDS',
  Barren = 'BARREN',
  BlackMarket = 'BLACK_MARKET',
  BreathableAtmosphere = 'BREATHABLE_ATMOSPHERE',
  Bureaucratic = 'BUREAUCRATIC',
  Canyons = 'CANYONS',
  CommonMetalDeposits = 'COMMON_METAL_DEPOSITS',
  CorrosiveAtmosphere = 'CORROSIVE_ATMOSPHERE',
  Corrupt = 'CORRUPT',
  CrushingGravity = 'CRUSHING_GRAVITY',
  DebrisCluster = 'DEBRIS_CLUSTER',
  DeepCraters = 'DEEP_CRATERS',
  DiverseLife = 'DIVERSE_LIFE',
  DrySeabeds = 'DRY_SEABEDS',
  ExplorationOutpost = 'EXPLORATION_OUTPOST',
  ExplosiveGases = 'EXPLOSIVE_GASES',
  ExtremePressure = 'EXTREME_PRESSURE',
  ExtremeTemperatures = 'EXTREME_TEMPERATURES',
  Fossils = 'FOSSILS',
  Frozen = 'FROZEN',
  HighTech = 'HIGH_TECH',
  HollowedInterior = 'HOLLOWED_INTERIOR',
  IceCrystals = 'ICE_CRYSTALS',
  Industrial = 'INDUSTRIAL',
  Jovian = 'JOVIAN',
  Jungle = 'JUNGLE',
  MagmaSeas = 'MAGMA_SEAS',
  Marketplace = 'MARKETPLACE',
  MegaStructures = 'MEGA_STRUCTURES',
  MethanePools = 'METHANE_POOLS',
  MicroGravityAnomalies = 'MICRO_GRAVITY_ANOMALIES',
  MilitaryBase = 'MILITARY_BASE',
  MineralDeposits = 'MINERAL_DEPOSITS',
  MutatedFlora = 'MUTATED_FLORA',
  Ocean = 'OCEAN',
  Outpost = 'OUTPOST',
  Overcrowded = 'OVERCROWDED',
  PerpetualDaylight = 'PERPETUAL_DAYLIGHT',
  PerpetualOvercast = 'PERPETUAL_OVERCAST',
  PirateBase = 'PIRATE_BASE',
  PreciousMetalDeposits = 'PRECIOUS_METAL_DEPOSITS',
  Radioactive = 'RADIOACTIVE',
  RareMetalDeposits = 'RARE_METAL_DEPOSITS',
  ResearchFacility = 'RESEARCH_FACILITY',
  Rocky = 'ROCKY',
  SaltFlats = 'SALT_FLATS',
  ScarceLife = 'SCARCE_LIFE',
  ScatteredSettlements = 'SCATTERED_SETTLEMENTS',
  ShallowCraters = 'SHALLOW_CRATERS',
  Shipyard = 'SHIPYARD',
  SprawlingCities = 'SPRAWLING_CITIES',
  Stripped = 'STRIPPED',
  StrongGravity = 'STRONG_GRAVITY',
  StrongMagnetosphere = 'STRONG_MAGNETOSPHERE',
  Supervolcanoes = 'SUPERVOLCANOES',
  SurveillanceOutpost = 'SURVEILLANCE_OUTPOST',
  Swamp = 'SWAMP',
  Temperate = 'TEMPERATE',
  Terraformed = 'TERRAFORMED',
  ThinAtmosphere = 'THIN_ATMOSPHERE',
  ToxicAtmosphere = 'TOXIC_ATMOSPHERE',
  TradingHub = 'TRADING_HUB',
  Uncharted = 'UNCHARTED',
  UnderConstruction = 'UNDER_CONSTRUCTION',
  UnstableComposition = 'UNSTABLE_COMPOSITION',
  VastRuins = 'VAST_RUINS',
  VibrantAuroras = 'VIBRANT_AURORAS',
  Volcanic = 'VOLCANIC',
  WeakGravity = 'WEAK_GRAVITY'
}

/**
 * WaypointType : The type of waypoint.
 * The type of waypoint.
 */
export enum WaypointType {
  ArtificialGravityWell = 'ARTIFICIAL_GRAVITY_WELL',
  Asteroid = 'ASTEROID',
  AsteroidBase = 'ASTEROID_BASE',
  AsteroidField = 'ASTEROID_FIELD',
  DebrisField = 'DEBRIS_FIELD',
  EngineeredAsteroid = 'ENGINEERED_ASTEROID',
  FuelStation = 'FUEL_STATION',
  GasGiant = 'GAS_GIANT',
  GravityWell = 'GRAVITY_WELL',
  JumpGate = 'JUMP_GATE',
  Moon = 'MOON',
  Nebula = 'NEBULA',
  OrbitalStation = 'ORBITAL_STATION',
  Planet = 'PLANET'
}

export type GetMainSiteDataQueryVariables = Exact<{ [key: string]: never; }>;


export type GetMainSiteDataQuery = { __typename?: 'QueryRoot', apiCounts: number, budget: { __typename?: 'BudgetInfo', currentFunds: number, ironReserve: number, reservedAmount: number, spendable: number }, runInfo: { __typename?: 'RunInfo', resetDate: string, nextResetDate: string, agent?: { __typename?: 'Agent', symbol: string, credits: number, shipCount: number } | null, headquartersSystem?: { __typename?: 'System', symbol: string, constructionMaterials: { __typename?: 'ConstructionMaterialPage', items: Array<{ __typename?: 'ConstructionMaterial', waypointSymbol: string, tradeSymbol: TradeSymbol, required: number, fulfilled: number }> } } | null }, systems: { __typename?: 'SystemPage', items: Array<{ __typename?: 'System', symbol: string, waypoints: { __typename?: 'WaypointPage', items: Array<{ __typename?: 'Waypoint', symbol: string, chartedBy?: string | null, hasMarketplace: boolean, hasShipyard: boolean }> } }> }, fleets: { __typename?: 'FleetPage', items: Array<{ __typename?: 'Fleet', id: number, systemSymbol: string, fleetType: FleetType, active: boolean, assignments: { __typename?: 'ShipAssignmentPage', items: Array<{ __typename?: 'ShipAssignment', id: number, priority: number, rangeMin: number, cargoMin: number, ship?: { __typename?: 'Ship', symbol: string } | null }> } }> }, shipAssignments: { __typename?: 'ShipAssignmentPage', items: Array<{ __typename?: 'ShipAssignment', id: number, fleetId: number, fleet?: { __typename?: 'Fleet', systemSymbol: string, fleetType: FleetType } | null }> }, ships: Array<{ __typename?: 'Ship', symbol: string, registrationRole: ShipRole, cooldownExpiration?: string | null, status: { __typename?: 'ShipStatus', assignmentId?: number | null, tempAssignmentId?: number | null, status:
        | { __typename: 'ChartingStatus' }
        | { __typename: 'ConstructionStatus' }
        | { __typename: 'ContractStatus' }
        | { __typename: 'ManuelStatus' }
        | { __typename: 'MiningStatus' }
        | { __typename: 'ScraperStatus' }
        | { __typename: 'TraderStatus' }
        | { __typename: 'TransferStatus' }
       }, nav: { __typename?: 'NavigationState', status: ShipNavStatus, systemSymbol: string }, cargo: { __typename?: 'CargoState', units: number } }>, chartManager: { __typename?: 'ChartManagerInfo', busy: boolean, channelState: { __typename?: 'ChannelInfo', usedCapacity: number, state: ChannelState } }, fleetManager: { __typename?: 'FleetManagerInfo', busy: boolean, channelState: { __typename?: 'ChannelInfo', usedCapacity: number, state: ChannelState } }, tradeManager: { __typename?: 'TradeManagerInfo', busy: boolean, channelState: { __typename?: 'ChannelInfo', usedCapacity: number, state: ChannelState } }, miningManager: { __typename?: 'MiningManagerInfo', busy: boolean, channelState: { __typename?: 'ChannelInfo', usedCapacity: number, state: ChannelState } }, contractManager: { __typename?: 'ContractManagerInfo', busy: boolean, channelState: { __typename?: 'ChannelInfo', usedCapacity: number, state: ChannelState } }, scrappingManager: { __typename?: 'ScrappingManagerInfo', busy: boolean, channelState: { __typename?: 'ChannelInfo', usedCapacity: number, state: ChannelState } }, constructionManager: { __typename?: 'ConstructionManagerInfo', busy: boolean, channelState: { __typename?: 'ChannelInfo', usedCapacity: number, state: ChannelState } } };

export type GetAllSystemsQueryVariables = Exact<{ [key: string]: never; }>;


export type GetAllSystemsQuery = { __typename?: 'QueryRoot', systems: { __typename?: 'SystemPage', items: Array<{ __typename?: 'System', symbol: string, constellation?: string | null, sectorSymbol: string, systemType: SystemType, x: number, y: number, populationDisabled: boolean, waypoints: { __typename?: 'WaypointPage', items: Array<{ __typename?: 'Waypoint', symbol: string, waypointType: WaypointType, hasShipyard: boolean, hasMarketplace: boolean }> }, fleets: { __typename?: 'FleetPage', items: Array<{ __typename?: 'Fleet', id: number, fleetType: FleetType, active: boolean }> }, ships: Array<{ __typename?: 'Ship', symbol: string }> }> } };

export type GetSystemMapDataQueryVariables = Exact<{ [key: string]: never; }>;


export type GetSystemMapDataQuery = { __typename?: 'QueryRoot', systems: { __typename?: 'SystemPage', items: Array<{ __typename?: 'System', symbol: string, constellation?: string | null, systemType: SystemType, x: number, y: number, populationDisabled: boolean, waypoints: { __typename?: 'WaypointPage', items: Array<{ __typename?: 'Waypoint', symbol: string, waypointType: WaypointType, hasShipyard: boolean, hasMarketplace: boolean, isUnderConstruction: boolean }> }, fleets: { __typename?: 'FleetPage', items: Array<{ __typename?: 'Fleet', id: number, fleetType: FleetType, active: boolean }> }, ships: Array<{ __typename?: 'Ship', symbol: string }> }> }, jumpConnections: { __typename?: 'GateConnPage', items: Array<{ __typename?: 'GateConn', underConstructionA: boolean, underConstructionB: boolean, pointASymbol: string, pointBSymbol: string, fromA: boolean, fromB: boolean }> } };

export type GetAllAgentsQueryVariables = Exact<{ [key: string]: never; }>;


export type GetAllAgentsQuery = { __typename?: 'QueryRoot', agents: Array<{ __typename?: 'Agent', symbol: string, credits: number, shipCount: number, startingFaction: string, headquarters: string, createdAt: string }> };

export type GetAgentHistoryQueryVariables = Exact<{
  agentSymbol: Scalars['String']['input'];
}>;


export type GetAgentHistoryQuery = { __typename?: 'QueryRoot', agent: { __typename?: 'Agent', symbol: string, credits: number, shipCount: number, accountId?: string | null, startingFaction: string, createdAt: string, headquarters: string, history: Array<{ __typename?: 'Agent', id: number, credits: number, shipCount: number, createdAt: string }> } };

export type GetAllSurveysQueryVariables = Exact<{ [key: string]: never; }>;


export type GetAllSurveysQuery = { __typename?: 'QueryRoot', surveys: { __typename?: 'SurveyPage', items: Array<{ __typename?: 'Survey', shipInfoBefore: number, updatedAt: string, shipInfoAfter: number, signature: string, size: SurveySize, waypointSymbol: string, deposits: Array<TradeSymbol>, exhaustedSince?: string | null, createdAt: string, expiration: string }> } };

export type GetSystemQueryVariables = Exact<{
  systemSymbol: Scalars['String']['input'];
}>;


export type GetSystemQuery = { __typename?: 'QueryRoot', system: { __typename?: 'System', symbol: string, sectorSymbol: string, constellation?: string | null, systemType: SystemType, x: number, y: number, populationDisabled: boolean, seenAgents: Array<{ __typename?: 'KnownAgent', symbol: string, count: number }>, fleets: { __typename?: 'FleetPage', items: Array<{ __typename?: 'Fleet', id: number, fleetType: FleetType, active: boolean, createdAt: string, updatedAt: string, assignments: { __typename?: 'ShipAssignmentPage', items: Array<{ __typename?: 'ShipAssignment', id: number, siphon: boolean, warpDrive: boolean, fleetId: number, priority: number, maxPurchasePrice: number, creditsThreshold: number, disabled: boolean, rangeMin: number, cargoMin: number, survey: boolean, extractor: boolean }> } }> }, chartTransactions: { __typename?: 'ChartTransactionPage', items: Array<{ __typename?: 'ChartTransaction', waypointSymbol: string, shipSymbol: string, totalPrice: number, timestamp: string }> }, shipyardShips: { __typename?: 'ShipyardShipPage', items: Array<{ __typename?: 'ShipyardShip', reactorQuality?: number | null, engineType: ShipEngineSymbol, engineQuality?: number | null, modules: Array<ShipModuleSymbol>, mounts: Array<ShipMountSymbol>, createdAt: string, waypointSymbol: string, shipType: ShipType, name: string, supply: SupplyLevel, activity?: ActivityLevel | null, purchasePrice: number, frameType: ShipFrameSymbol, frameQuality?: number | null, reactorType: ShipReactorSymbol }> }, marketTrades: { __typename?: 'MarketTradePage', items: Array<{ __typename?: 'MarketTrade', waypointSymbol: string, symbol: TradeSymbol, createdAt: string, type: Type, tradeSymbolInfo: { __typename?: 'TradeSymbolInfo', symbol: TradeSymbol, requires: { __typename?: 'TradeSymbolInfoPage', items: Array<{ __typename?: 'TradeSymbolInfo', symbol: TradeSymbol }> }, requiredBy: { __typename?: 'TradeSymbolInfoPage', items: Array<{ __typename?: 'TradeSymbolInfo', symbol: TradeSymbol }> } }, marketTradeGood?: { __typename?: 'MarketTradeGood', symbol: TradeSymbol, waypointSymbol: string, type: Type, tradeVolume: number, supply: SupplyLevel, activity?: ActivityLevel | null, purchasePrice: number, sellPrice: number, createdAt: string } | null }> }, constructionMaterials: { __typename?: 'ConstructionMaterialPage', items: Array<{ __typename?: 'ConstructionMaterial', waypointSymbol: string, tradeSymbol: TradeSymbol, required: number, fulfilled: number, marketTransactionSummary: { __typename?: 'TransactionSummary', expenses?: number | null, purchaseUnits?: number | null, purchaseTransactions?: number | null } }> }, jumpGateConnections: { __typename?: 'JumpGateConnectionPage', items: Array<{ __typename?: 'JumpGateConnection', from: string, to: string }> }, waypoints: { __typename?: 'WaypointPage', items: Array<{ __typename?: 'Waypoint', symbol: string, faction?: string | null, modifiers: Array<WaypointModifierSymbol>, chartedBy?: string | null, chartedOn?: string | null, hasShipyard: boolean, hasMarketplace: boolean, x: number, y: number, lastScrap?: string | null, nextScrap?: string | null, waypointType: WaypointType, traits: Array<WaypointTraitSymbol>, isUnderConstruction: boolean, orbitals: Array<string>, orbits?: string | null, marketTrades: { __typename?: 'MarketTradePage', items: Array<{ __typename?: 'MarketTrade', symbol: TradeSymbol, type: Type, tradeSymbolInfo: { __typename?: 'TradeSymbolInfo', symbol: TradeSymbol, requires: { __typename?: 'TradeSymbolInfoPage', items: Array<{ __typename?: 'TradeSymbolInfo', symbol: TradeSymbol }> }, requiredBy: { __typename?: 'TradeSymbolInfoPage', items: Array<{ __typename?: 'TradeSymbolInfo', symbol: TradeSymbol }> } }, marketTradeGood?: { __typename?: 'MarketTradeGood', tradeVolume: number, supply: SupplyLevel, activity?: ActivityLevel | null, purchasePrice: number, sellPrice: number } | null }> }, shipyardShips: { __typename?: 'ShipyardShipPage', items: Array<{ __typename?: 'ShipyardShip', shipType: ShipType, supply: SupplyLevel, activity?: ActivityLevel | null, purchasePrice: number }> } }> }, marketTransactions: { __typename?: 'MarketTransactionPage', items: Array<{ __typename?: 'MarketTransaction', id: number, waypointSymbol: string, shipSymbol: string, tradeSymbol: TradeSymbol, type: MarketTransactionType, units: number, pricePerUnit: number, totalPrice: number, timestamp: string, contract_id?: string | null }> }, shipyardTransactions: { __typename?: 'ShipyardTransactionPage', items: Array<{ __typename?: 'ShipyardTransaction', id: number, waypointSymbol: string, shipType: ShipType, price: number, agentSymbol: string, timestamp: string }> }, contractDeliveries: { __typename?: 'ContractDeliveryPage', items: Array<{ __typename?: 'ContractDelivery', contractId: string, tradeSymbol: TradeSymbol, destinationSymbol: string, unitsRequired: number, unitsFulfilled: number, contract?: { __typename?: 'Contract', id: string, createdAt: string, reservedFund?: number | null, factionSymbol: string, contractType: ContractType, accepted: boolean, onFulfilled: number, deadline: string, marketTransactionSummary: { __typename?: 'TransactionSummary', sum?: number | null, expenses?: number | null, income?: number | null, units?: number | null, purchaseUnits?: number | null, sellUnits?: number | null, purchaseTransactions?: number | null, sellTransactions?: number | null } } | null }> }, tradeRoutes: { __typename?: 'TradeRoutePage', items: Array<{ __typename?: 'TradeRoute', id: number, reservedFund?: number | null, symbol: TradeSymbol, shipSymbol: string, PurchaseWaypointSymbol: string, SellWaypointSymbol: string, status: ShipmentStatus, tradeVolume: number, predictedPurchasePrice: number, predictedSellPrice: number, marketTransactionSummary: { __typename?: 'TransactionSummary', sum?: number | null, expenses?: number | null, income?: number | null, units?: number | null, purchaseUnits?: number | null, sellUnits?: number | null, purchaseTransactions?: number | null, sellTransactions?: number | null } }> }, ships: Array<{ __typename?: 'Ship', symbol: string, nav: { __typename?: 'NavigationState', waypointSymbol: string, status: ShipNavStatus }, fuel: { __typename?: 'FuelState', capacity: number }, cargo: { __typename?: 'CargoState', capacity: number }, status: { __typename?: 'ShipStatus', assignmentId?: number | null, fleetId?: number | null, tempAssignmentId?: number | null, tempFleetId?: number | null, status:
          | { __typename: 'ChartingStatus' }
          | { __typename: 'ConstructionStatus' }
          | { __typename: 'ContractStatus' }
          | { __typename: 'ManuelStatus' }
          | { __typename: 'MiningStatus' }
          | { __typename: 'ScraperStatus' }
          | { __typename: 'TraderStatus' }
          | { __typename: 'TransferStatus' }
         } }> } };

export type GetChartTransactionsQueryVariables = Exact<{ [key: string]: never; }>;


export type GetChartTransactionsQuery = { __typename?: 'QueryRoot', chartTransactions: { __typename?: 'ChartTransactionPage', items: Array<{ __typename?: 'ChartTransaction', waypointSymbol: string, shipSymbol: string, totalPrice: number, timestamp: string, waypoint?: { __typename?: 'Waypoint', symbol: string, waypointType: WaypointType, traits: Array<WaypointTraitSymbol> } | null }> } };


export const GetMainSiteDataDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"query","name":{"kind":"Name","value":"GetMainSiteData"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"apiCounts"}},{"kind":"Field","name":{"kind":"Name","value":"budget"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"currentFunds"}},{"kind":"Field","name":{"kind":"Name","value":"ironReserve"}},{"kind":"Field","name":{"kind":"Name","value":"reservedAmount"}},{"kind":"Field","name":{"kind":"Name","value":"spendable"}}]}},{"kind":"Field","name":{"kind":"Name","value":"runInfo"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"resetDate"}},{"kind":"Field","name":{"kind":"Name","value":"nextResetDate"}},{"kind":"Field","name":{"kind":"Name","value":"agent"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"symbol"}},{"kind":"Field","name":{"kind":"Name","value":"credits"}},{"kind":"Field","name":{"kind":"Name","value":"shipCount"}}]}},{"kind":"Field","name":{"kind":"Name","value":"headquartersSystem"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"symbol"}},{"kind":"Field","name":{"kind":"Name","value":"constructionMaterials"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"items"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"waypointSymbol"}},{"kind":"Field","name":{"kind":"Name","value":"tradeSymbol"}},{"kind":"Field","name":{"kind":"Name","value":"required"}},{"kind":"Field","name":{"kind":"Name","value":"fulfilled"}}]}}]}}]}}]}},{"kind":"Field","name":{"kind":"Name","value":"systems"},"arguments":[{"kind":"Argument","name":{"kind":"Name","value":"onlyWithFleetsOrShips"},"value":{"kind":"BooleanValue","value":true}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"items"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"symbol"}},{"kind":"Field","name":{"kind":"Name","value":"waypoints"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"items"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"symbol"}},{"kind":"Field","name":{"kind":"Name","value":"chartedBy"}},{"kind":"Field","name":{"kind":"Name","value":"hasMarketplace"}},{"kind":"Field","name":{"kind":"Name","value":"hasShipyard"}}]}}]}}]}}]}},{"kind":"Field","name":{"kind":"Name","value":"fleets"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"items"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"systemSymbol"}},{"kind":"Field","name":{"kind":"Name","value":"fleetType"}},{"kind":"Field","name":{"kind":"Name","value":"active"}},{"kind":"Field","name":{"kind":"Name","value":"assignments"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"items"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"priority"}},{"kind":"Field","name":{"kind":"Name","value":"rangeMin"}},{"kind":"Field","name":{"kind":"Name","value":"cargoMin"}},{"kind":"Field","name":{"kind":"Name","value":"ship"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"symbol"}}]}}]}}]}}]}}]}},{"kind":"Field","name":{"kind":"Name","value":"shipAssignments"},"arguments":[{"kind":"Argument","name":{"kind":"Name","value":"by"},"value":{"kind":"ObjectValue","fields":[{"kind":"ObjectField","name":{"kind":"Name","value":"open"},"value":{"kind":"BooleanValue","value":true}}]}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"items"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"fleetId"}},{"kind":"Field","name":{"kind":"Name","value":"fleet"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"systemSymbol"}},{"kind":"Field","name":{"kind":"Name","value":"fleetType"}}]}}]}}]}},{"kind":"Field","name":{"kind":"Name","value":"ships"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"symbol"}},{"kind":"Field","name":{"kind":"Name","value":"registrationRole"}},{"kind":"Field","name":{"kind":"Name","value":"status"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"assignmentId"}},{"kind":"Field","name":{"kind":"Name","value":"tempAssignmentId"}},{"kind":"Field","name":{"kind":"Name","value":"status"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"__typename"}}]}}]}},{"kind":"Field","name":{"kind":"Name","value":"nav"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"status"}},{"kind":"Field","name":{"kind":"Name","value":"systemSymbol"}}]}},{"kind":"Field","name":{"kind":"Name","value":"cargo"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"units"}}]}},{"kind":"Field","name":{"kind":"Name","value":"cooldownExpiration"}}]}},{"kind":"Field","name":{"kind":"Name","value":"chartManager"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"busy"}},{"kind":"Field","name":{"kind":"Name","value":"channelState"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"usedCapacity"}},{"kind":"Field","name":{"kind":"Name","value":"state"}}]}}]}},{"kind":"Field","name":{"kind":"Name","value":"fleetManager"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"busy"}},{"kind":"Field","name":{"kind":"Name","value":"channelState"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"usedCapacity"}},{"kind":"Field","name":{"kind":"Name","value":"state"}}]}}]}},{"kind":"Field","name":{"kind":"Name","value":"tradeManager"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"busy"}},{"kind":"Field","name":{"kind":"Name","value":"channelState"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"usedCapacity"}},{"kind":"Field","name":{"kind":"Name","value":"state"}}]}}]}},{"kind":"Field","name":{"kind":"Name","value":"miningManager"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"busy"}},{"kind":"Field","name":{"kind":"Name","value":"channelState"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"usedCapacity"}},{"kind":"Field","name":{"kind":"Name","value":"state"}}]}}]}},{"kind":"Field","name":{"kind":"Name","value":"contractManager"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"busy"}},{"kind":"Field","name":{"kind":"Name","value":"channelState"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"usedCapacity"}},{"kind":"Field","name":{"kind":"Name","value":"state"}}]}}]}},{"kind":"Field","name":{"kind":"Name","value":"scrappingManager"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"busy"}},{"kind":"Field","name":{"kind":"Name","value":"channelState"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"usedCapacity"}},{"kind":"Field","name":{"kind":"Name","value":"state"}}]}}]}},{"kind":"Field","name":{"kind":"Name","value":"constructionManager"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"busy"}},{"kind":"Field","name":{"kind":"Name","value":"channelState"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"usedCapacity"}},{"kind":"Field","name":{"kind":"Name","value":"state"}}]}}]}}]}}]} as unknown as DocumentNode<GetMainSiteDataQuery, GetMainSiteDataQueryVariables>;
export const GetAllSystemsDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"query","name":{"kind":"Name","value":"GetAllSystems"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"systems"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"items"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"symbol"}},{"kind":"Field","name":{"kind":"Name","value":"constellation"}},{"kind":"Field","name":{"kind":"Name","value":"sectorSymbol"}},{"kind":"Field","name":{"kind":"Name","value":"systemType"}},{"kind":"Field","name":{"kind":"Name","value":"x"}},{"kind":"Field","name":{"kind":"Name","value":"y"}},{"kind":"Field","name":{"kind":"Name","value":"populationDisabled"}},{"kind":"Field","name":{"kind":"Name","value":"waypoints"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"items"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"symbol"}},{"kind":"Field","name":{"kind":"Name","value":"waypointType"}},{"kind":"Field","name":{"kind":"Name","value":"hasShipyard"}},{"kind":"Field","name":{"kind":"Name","value":"hasMarketplace"}}]}}]}},{"kind":"Field","name":{"kind":"Name","value":"fleets"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"items"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"fleetType"}},{"kind":"Field","name":{"kind":"Name","value":"active"}}]}}]}},{"kind":"Field","name":{"kind":"Name","value":"ships"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"symbol"}}]}}]}}]}}]}}]} as unknown as DocumentNode<GetAllSystemsQuery, GetAllSystemsQueryVariables>;
export const GetSystemMapDataDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"query","name":{"kind":"Name","value":"GetSystemMapData"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"systems"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"items"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"symbol"}},{"kind":"Field","name":{"kind":"Name","value":"constellation"}},{"kind":"Field","name":{"kind":"Name","value":"systemType"}},{"kind":"Field","name":{"kind":"Name","value":"x"}},{"kind":"Field","name":{"kind":"Name","value":"y"}},{"kind":"Field","name":{"kind":"Name","value":"populationDisabled"}},{"kind":"Field","name":{"kind":"Name","value":"waypoints"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"items"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"symbol"}},{"kind":"Field","name":{"kind":"Name","value":"waypointType"}},{"kind":"Field","name":{"kind":"Name","value":"hasShipyard"}},{"kind":"Field","name":{"kind":"Name","value":"hasMarketplace"}},{"kind":"Field","name":{"kind":"Name","value":"isUnderConstruction"}}]}}]}},{"kind":"Field","name":{"kind":"Name","value":"fleets"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"items"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"fleetType"}},{"kind":"Field","name":{"kind":"Name","value":"active"}}]}}]}},{"kind":"Field","name":{"kind":"Name","value":"ships"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"symbol"}}]}}]}}]}},{"kind":"Field","name":{"kind":"Name","value":"jumpConnections"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"items"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"underConstructionA"}},{"kind":"Field","name":{"kind":"Name","value":"underConstructionB"}},{"kind":"Field","name":{"kind":"Name","value":"pointASymbol"}},{"kind":"Field","name":{"kind":"Name","value":"pointBSymbol"}},{"kind":"Field","name":{"kind":"Name","value":"fromA"}},{"kind":"Field","name":{"kind":"Name","value":"fromB"}}]}}]}}]}}]} as unknown as DocumentNode<GetSystemMapDataQuery, GetSystemMapDataQueryVariables>;
export const GetAllAgentsDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"query","name":{"kind":"Name","value":"GetAllAgents"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"agents"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"symbol"}},{"kind":"Field","name":{"kind":"Name","value":"credits"}},{"kind":"Field","name":{"kind":"Name","value":"shipCount"}},{"kind":"Field","name":{"kind":"Name","value":"startingFaction"}},{"kind":"Field","name":{"kind":"Name","value":"headquarters"}},{"kind":"Field","name":{"kind":"Name","value":"createdAt"}}]}}]}}]} as unknown as DocumentNode<GetAllAgentsQuery, GetAllAgentsQueryVariables>;
export const GetAgentHistoryDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"query","name":{"kind":"Name","value":"GetAgentHistory"},"variableDefinitions":[{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"agentSymbol"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"String"}}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"agent"},"arguments":[{"kind":"Argument","name":{"kind":"Name","value":"symbol"},"value":{"kind":"Variable","name":{"kind":"Name","value":"agentSymbol"}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"symbol"}},{"kind":"Field","name":{"kind":"Name","value":"credits"}},{"kind":"Field","name":{"kind":"Name","value":"shipCount"}},{"kind":"Field","name":{"kind":"Name","value":"accountId"}},{"kind":"Field","name":{"kind":"Name","value":"startingFaction"}},{"kind":"Field","name":{"kind":"Name","value":"createdAt"}},{"kind":"Field","name":{"kind":"Name","value":"headquarters"}},{"kind":"Field","name":{"kind":"Name","value":"history"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"credits"}},{"kind":"Field","name":{"kind":"Name","value":"shipCount"}},{"kind":"Field","name":{"kind":"Name","value":"createdAt"}}]}}]}}]}}]} as unknown as DocumentNode<GetAgentHistoryQuery, GetAgentHistoryQueryVariables>;
export const GetAllSurveysDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"query","name":{"kind":"Name","value":"GetAllSurveys"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"surveys"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"items"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"shipInfoBefore"}},{"kind":"Field","name":{"kind":"Name","value":"updatedAt"}},{"kind":"Field","name":{"kind":"Name","value":"shipInfoAfter"}},{"kind":"Field","name":{"kind":"Name","value":"signature"}},{"kind":"Field","name":{"kind":"Name","value":"signature"}},{"kind":"Field","name":{"kind":"Name","value":"size"}},{"kind":"Field","name":{"kind":"Name","value":"waypointSymbol"}},{"kind":"Field","name":{"kind":"Name","value":"deposits"}},{"kind":"Field","name":{"kind":"Name","value":"exhaustedSince"}},{"kind":"Field","name":{"kind":"Name","value":"createdAt"}},{"kind":"Field","name":{"kind":"Name","value":"expiration"}}]}}]}}]}}]} as unknown as DocumentNode<GetAllSurveysQuery, GetAllSurveysQueryVariables>;
export const GetSystemDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"query","name":{"kind":"Name","value":"GetSystem"},"variableDefinitions":[{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"systemSymbol"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"String"}}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"system"},"arguments":[{"kind":"Argument","name":{"kind":"Name","value":"symbol"},"value":{"kind":"Variable","name":{"kind":"Name","value":"systemSymbol"}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"symbol"}},{"kind":"Field","name":{"kind":"Name","value":"sectorSymbol"}},{"kind":"Field","name":{"kind":"Name","value":"constellation"}},{"kind":"Field","name":{"kind":"Name","value":"systemType"}},{"kind":"Field","name":{"kind":"Name","value":"x"}},{"kind":"Field","name":{"kind":"Name","value":"y"}},{"kind":"Field","name":{"kind":"Name","value":"populationDisabled"}},{"kind":"Field","name":{"kind":"Name","value":"seenAgents"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"symbol"}},{"kind":"Field","name":{"kind":"Name","value":"count"}}]}},{"kind":"Field","name":{"kind":"Name","value":"fleets"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"items"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"fleetType"}},{"kind":"Field","name":{"kind":"Name","value":"active"}},{"kind":"Field","name":{"kind":"Name","value":"assignments"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"items"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"siphon"}},{"kind":"Field","name":{"kind":"Name","value":"warpDrive"}},{"kind":"Field","name":{"kind":"Name","value":"fleetId"}},{"kind":"Field","name":{"kind":"Name","value":"priority"}},{"kind":"Field","name":{"kind":"Name","value":"maxPurchasePrice"}},{"kind":"Field","name":{"kind":"Name","value":"creditsThreshold"}},{"kind":"Field","name":{"kind":"Name","value":"disabled"}},{"kind":"Field","name":{"kind":"Name","value":"rangeMin"}},{"kind":"Field","name":{"kind":"Name","value":"cargoMin"}},{"kind":"Field","name":{"kind":"Name","value":"survey"}},{"kind":"Field","name":{"kind":"Name","value":"extractor"}}]}}]}},{"kind":"Field","name":{"kind":"Name","value":"createdAt"}},{"kind":"Field","name":{"kind":"Name","value":"updatedAt"}}]}}]}},{"kind":"Field","name":{"kind":"Name","value":"chartTransactions"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"items"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"waypointSymbol"}},{"kind":"Field","name":{"kind":"Name","value":"shipSymbol"}},{"kind":"Field","name":{"kind":"Name","value":"totalPrice"}},{"kind":"Field","name":{"kind":"Name","value":"timestamp"}}]}}]}},{"kind":"Field","name":{"kind":"Name","value":"shipyardShips"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"items"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"reactorQuality"}},{"kind":"Field","name":{"kind":"Name","value":"engineType"}},{"kind":"Field","name":{"kind":"Name","value":"engineQuality"}},{"kind":"Field","name":{"kind":"Name","value":"modules"}},{"kind":"Field","name":{"kind":"Name","value":"mounts"}},{"kind":"Field","name":{"kind":"Name","value":"createdAt"}},{"kind":"Field","name":{"kind":"Name","value":"waypointSymbol"}},{"kind":"Field","name":{"kind":"Name","value":"shipType"}},{"kind":"Field","name":{"kind":"Name","value":"name"}},{"kind":"Field","name":{"kind":"Name","value":"supply"}},{"kind":"Field","name":{"kind":"Name","value":"activity"}},{"kind":"Field","name":{"kind":"Name","value":"purchasePrice"}},{"kind":"Field","name":{"kind":"Name","value":"frameType"}},{"kind":"Field","name":{"kind":"Name","value":"frameQuality"}},{"kind":"Field","name":{"kind":"Name","value":"reactorType"}}]}}]}},{"kind":"Field","name":{"kind":"Name","value":"marketTrades"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"items"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"waypointSymbol"}},{"kind":"Field","name":{"kind":"Name","value":"symbol"}},{"kind":"Field","name":{"kind":"Name","value":"createdAt"}},{"kind":"Field","name":{"kind":"Name","value":"type"}},{"kind":"Field","name":{"kind":"Name","value":"tradeSymbolInfo"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"symbol"}},{"kind":"Field","name":{"kind":"Name","value":"requires"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"items"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"symbol"}}]}}]}},{"kind":"Field","name":{"kind":"Name","value":"requiredBy"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"items"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"symbol"}}]}}]}}]}},{"kind":"Field","name":{"kind":"Name","value":"marketTradeGood"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"symbol"}},{"kind":"Field","name":{"kind":"Name","value":"waypointSymbol"}},{"kind":"Field","name":{"kind":"Name","value":"type"}},{"kind":"Field","name":{"kind":"Name","value":"tradeVolume"}},{"kind":"Field","name":{"kind":"Name","value":"supply"}},{"kind":"Field","name":{"kind":"Name","value":"activity"}},{"kind":"Field","name":{"kind":"Name","value":"purchasePrice"}},{"kind":"Field","name":{"kind":"Name","value":"sellPrice"}},{"kind":"Field","name":{"kind":"Name","value":"createdAt"}}]}}]}}]}},{"kind":"Field","name":{"kind":"Name","value":"constructionMaterials"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"items"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"waypointSymbol"}},{"kind":"Field","name":{"kind":"Name","value":"tradeSymbol"}},{"kind":"Field","name":{"kind":"Name","value":"required"}},{"kind":"Field","name":{"kind":"Name","value":"fulfilled"}},{"kind":"Field","name":{"kind":"Name","value":"marketTransactionSummary"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"expenses"}},{"kind":"Field","name":{"kind":"Name","value":"purchaseUnits"}},{"kind":"Field","name":{"kind":"Name","value":"purchaseTransactions"}}]}}]}}]}},{"kind":"Field","name":{"kind":"Name","value":"jumpGateConnections"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"items"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"from"}},{"kind":"Field","name":{"kind":"Name","value":"to"}}]}}]}},{"kind":"Field","name":{"kind":"Name","value":"waypoints"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"items"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"symbol"}},{"kind":"Field","name":{"kind":"Name","value":"faction"}},{"kind":"Field","name":{"kind":"Name","value":"modifiers"}},{"kind":"Field","name":{"kind":"Name","value":"chartedBy"}},{"kind":"Field","name":{"kind":"Name","value":"chartedOn"}},{"kind":"Field","name":{"kind":"Name","value":"hasShipyard"}},{"kind":"Field","name":{"kind":"Name","value":"hasMarketplace"}},{"kind":"Field","name":{"kind":"Name","value":"x"}},{"kind":"Field","name":{"kind":"Name","value":"y"}},{"kind":"Field","name":{"kind":"Name","value":"lastScrap"}},{"kind":"Field","name":{"kind":"Name","value":"nextScrap"}},{"kind":"Field","name":{"kind":"Name","value":"waypointType"}},{"kind":"Field","name":{"kind":"Name","value":"traits"}},{"kind":"Field","name":{"kind":"Name","value":"isUnderConstruction"}},{"kind":"Field","name":{"kind":"Name","value":"orbitals"}},{"kind":"Field","name":{"kind":"Name","value":"orbits"}},{"kind":"Field","name":{"kind":"Name","value":"marketTrades"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"items"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"symbol"}},{"kind":"Field","name":{"kind":"Name","value":"type"}},{"kind":"Field","name":{"kind":"Name","value":"tradeSymbolInfo"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"symbol"}},{"kind":"Field","name":{"kind":"Name","value":"requires"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"items"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"symbol"}}]}}]}},{"kind":"Field","name":{"kind":"Name","value":"requiredBy"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"items"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"symbol"}}]}}]}}]}},{"kind":"Field","name":{"kind":"Name","value":"marketTradeGood"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"tradeVolume"}},{"kind":"Field","name":{"kind":"Name","value":"supply"}},{"kind":"Field","name":{"kind":"Name","value":"activity"}},{"kind":"Field","name":{"kind":"Name","value":"purchasePrice"}},{"kind":"Field","name":{"kind":"Name","value":"sellPrice"}}]}}]}}]}},{"kind":"Field","name":{"kind":"Name","value":"shipyardShips"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"items"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"shipType"}},{"kind":"Field","name":{"kind":"Name","value":"supply"}},{"kind":"Field","name":{"kind":"Name","value":"activity"}},{"kind":"Field","name":{"kind":"Name","value":"purchasePrice"}}]}}]}}]}}]}},{"kind":"Field","name":{"kind":"Name","value":"marketTransactions"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"items"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"waypointSymbol"}},{"kind":"Field","name":{"kind":"Name","value":"shipSymbol"}},{"kind":"Field","name":{"kind":"Name","value":"tradeSymbol"}},{"kind":"Field","name":{"kind":"Name","value":"type"}},{"kind":"Field","name":{"kind":"Name","value":"units"}},{"kind":"Field","name":{"kind":"Name","value":"pricePerUnit"}},{"kind":"Field","name":{"kind":"Name","value":"totalPrice"}},{"kind":"Field","name":{"kind":"Name","value":"timestamp"}},{"kind":"Field","name":{"kind":"Name","value":"contract_id"}}]}}]}},{"kind":"Field","name":{"kind":"Name","value":"shipyardTransactions"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"items"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"waypointSymbol"}},{"kind":"Field","name":{"kind":"Name","value":"shipType"}},{"kind":"Field","name":{"kind":"Name","value":"price"}},{"kind":"Field","name":{"kind":"Name","value":"agentSymbol"}},{"kind":"Field","name":{"kind":"Name","value":"timestamp"}}]}}]}},{"kind":"Field","name":{"kind":"Name","value":"contractDeliveries"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"items"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"contractId"}},{"kind":"Field","name":{"kind":"Name","value":"tradeSymbol"}},{"kind":"Field","name":{"kind":"Name","value":"destinationSymbol"}},{"kind":"Field","name":{"kind":"Name","value":"unitsRequired"}},{"kind":"Field","name":{"kind":"Name","value":"unitsFulfilled"}},{"kind":"Field","name":{"kind":"Name","value":"contract"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"createdAt"}},{"kind":"Field","name":{"kind":"Name","value":"reservedFund"}},{"kind":"Field","name":{"kind":"Name","value":"factionSymbol"}},{"kind":"Field","name":{"kind":"Name","value":"contractType"}},{"kind":"Field","name":{"kind":"Name","value":"accepted"}},{"kind":"Field","name":{"kind":"Name","value":"onFulfilled"}},{"kind":"Field","name":{"kind":"Name","value":"deadline"}},{"kind":"Field","name":{"kind":"Name","value":"marketTransactionSummary"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"sum"}},{"kind":"Field","name":{"kind":"Name","value":"expenses"}},{"kind":"Field","name":{"kind":"Name","value":"income"}},{"kind":"Field","name":{"kind":"Name","value":"units"}},{"kind":"Field","name":{"kind":"Name","value":"purchaseUnits"}},{"kind":"Field","name":{"kind":"Name","value":"sellUnits"}},{"kind":"Field","name":{"kind":"Name","value":"purchaseTransactions"}},{"kind":"Field","name":{"kind":"Name","value":"sellTransactions"}}]}}]}}]}}]}},{"kind":"Field","name":{"kind":"Name","value":"tradeRoutes"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"items"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"reservedFund"}},{"kind":"Field","name":{"kind":"Name","value":"marketTransactionSummary"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"sum"}},{"kind":"Field","name":{"kind":"Name","value":"expenses"}},{"kind":"Field","name":{"kind":"Name","value":"income"}},{"kind":"Field","name":{"kind":"Name","value":"units"}},{"kind":"Field","name":{"kind":"Name","value":"purchaseUnits"}},{"kind":"Field","name":{"kind":"Name","value":"sellUnits"}},{"kind":"Field","name":{"kind":"Name","value":"purchaseTransactions"}},{"kind":"Field","name":{"kind":"Name","value":"sellTransactions"}}]}},{"kind":"Field","name":{"kind":"Name","value":"symbol"}},{"kind":"Field","name":{"kind":"Name","value":"shipSymbol"}},{"kind":"Field","name":{"kind":"Name","value":"PurchaseWaypointSymbol"}},{"kind":"Field","name":{"kind":"Name","value":"SellWaypointSymbol"}},{"kind":"Field","name":{"kind":"Name","value":"status"}},{"kind":"Field","name":{"kind":"Name","value":"tradeVolume"}},{"kind":"Field","name":{"kind":"Name","value":"predictedPurchasePrice"}},{"kind":"Field","name":{"kind":"Name","value":"predictedSellPrice"}}]}}]}},{"kind":"Field","name":{"kind":"Name","value":"ships"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"symbol"}},{"kind":"Field","name":{"kind":"Name","value":"nav"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"waypointSymbol"}},{"kind":"Field","name":{"kind":"Name","value":"status"}}]}},{"kind":"Field","name":{"kind":"Name","value":"fuel"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"capacity"}}]}},{"kind":"Field","name":{"kind":"Name","value":"cargo"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"capacity"}}]}},{"kind":"Field","name":{"kind":"Name","value":"status"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"assignmentId"}},{"kind":"Field","name":{"kind":"Name","value":"fleetId"}},{"kind":"Field","name":{"kind":"Name","value":"tempAssignmentId"}},{"kind":"Field","name":{"kind":"Name","value":"tempFleetId"}},{"kind":"Field","name":{"kind":"Name","value":"status"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"__typename"}}]}}]}}]}}]}}]}}]} as unknown as DocumentNode<GetSystemQuery, GetSystemQueryVariables>;
export const GetChartTransactionsDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"query","name":{"kind":"Name","value":"GetChartTransactions"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"chartTransactions"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"items"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"waypointSymbol"}},{"kind":"Field","name":{"kind":"Name","value":"shipSymbol"}},{"kind":"Field","name":{"kind":"Name","value":"totalPrice"}},{"kind":"Field","name":{"kind":"Name","value":"timestamp"}},{"kind":"Field","name":{"kind":"Name","value":"waypoint"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"symbol"}},{"kind":"Field","name":{"kind":"Name","value":"waypointType"}},{"kind":"Field","name":{"kind":"Name","value":"traits"}}]}}]}}]}}]}}]} as unknown as DocumentNode<GetChartTransactionsQuery, GetChartTransactionsQueryVariables>;