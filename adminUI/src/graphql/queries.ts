import { graphql } from "../gql/gql";

export const GET_MAIN_SITE_DATA = graphql(/* GraphQL */ `
  query GetMainSiteData {
    apiCounts
    runInfo {
      resetDate
      nextResetDate
      agent {
        symbol
        credits
        shipCount
      }
      headquartersSystem {
        symbol
        constructionMaterials {
          waypointSymbol
          tradeSymbol
          required
          fulfilled
        }
      }
    }
    systems(onlyWithFleetsOrShips: true) {
      symbol
      waypoints {
        symbol
        chartedBy
        hasMarketplace
        hasShipyard
      }
    }
    fleets {
      id
      systemSymbol
      fleetType
      active
      assignments {
        id
        priority
        rangeMin
        cargoMin
        ship {
          symbol
        }
      }
    }
    shipAssignments(by: { open: true }) {
      id
      fleetId
      fleet {
        systemSymbol
        fleetType
      }
    }
    ships {
      symbol
      registrationRole
      status {
        assignmentId
        tempAssignmentId
        status {
          __typename
        }
      }
      nav {
        status
        systemSymbol
      }
      cargo {
        units
      }
      cooldownExpiration
    }
    chartManager {
      busy
      channelState {
        usedCapacity
        state
      }
    }
    fleetManager {
      busy
      channelState {
        usedCapacity
        state
      }
    }
    tradeManager {
      busy
      channelState {
        usedCapacity
        state
      }
    }
    miningManager {
      busy
      channelState {
        usedCapacity
        state
      }
    }
    contractManager {
      busy
      channelState {
        usedCapacity
        state
      }
    }
    scrappingManager {
      busy
      channelState {
        usedCapacity
        state
      }
    }
    constructionManager {
      busy
      channelState {
        usedCapacity
        state
      }
    }
  }
`);

export const GET_ALL_SYSTEMS = graphql(/* GraphQL */ `
  query GetAllSystems {
    systems {
      symbol
      constellation
      sectorSymbol
      systemType
      x
      y
      populationDisabled
      waypoints {
        symbol
        waypointType
        hasShipyard
        hasMarketplace
      }
      fleets {
        id
        fleetType
        active
      }
      ships {
        symbol
      }
    }
  }
`);

export const GET_SYSTEM_MAP_DATA = graphql(/* GraphQL */ `
  query GetSystemMapData {
    systems {
      symbol
      constellation
      systemType
      x
      y
      populationDisabled
      waypoints {
        symbol
        waypointType
        hasShipyard
        hasMarketplace
        isUnderConstruction
      }
      fleets {
        id
        fleetType
        active
      }
      ships {
        symbol
      }
    }
    jumpConnections {
      underConstructionA
      underConstructionB
      pointASymbol
      pointBSymbol
      fromA
      fromB
    }
  }
`);

export const GET_ALL_AGENTS = graphql(/* GraphQL */ `
  query GetAllAgents {
    agents {
      symbol
      credits
      shipCount
      startingFaction
      headquarters
      createdAt
    }
  }
`);

export const GET_AGENT_HISTORY = graphql(/* GraphQL */ `
  query GetAgentHistory($agentSymbol: String!) {
    agent(symbol: $agentSymbol) {
      symbol
      credits
      shipCount
      accountId
      startingFaction
      createdAt
      headquarters
      history {
        id
        credits
        shipCount
        createdAt
      }
    }
  }
`);

export const GET_ALL_SURVEYS = graphql(/* GraphQL */ `
  query GetAllSurveys {
    surveys {
      shipInfoBefore
      updatedAt
      shipInfoAfter
      signature
      signature
      size
      waypointSymbol
      deposits
      exhaustedSince
      createdAt
      expiration
    }
  }
`);

export const GET_SYSTEM = graphql(/* GraphQL */ `
  query GetSystem($systemSymbol: String!) {
    system(symbol: $systemSymbol) {
      symbol
      sectorSymbol
      constellation
      systemType
      x
      y
      populationDisabled
      seenAgents {
        symbol
        count
      }
      fleets {
        id
        fleetType
        active
        assignments {
          id
          siphon
          warpDrive
          fleetId
          priority
          maxPurchasePrice
          creditsThreshold
          disabled
          rangeMin
          cargoMin
          survey
          extractor
        }
        createdAt
        updatedAt
      }
      chartTransactions {
        waypointSymbol
        shipSymbol
        totalPrice
        timestamp
      }
      shipyardShips {
        reactorQuality
        engineType
        engineQuality
        modules
        mounts
        createdAt
        waypointSymbol
        shipType
        name
        supply
        activity
        purchasePrice
        frameType
        frameQuality
        reactorType
      }
      shipyardShipTypes {
        shipType
        createdAt
        shipyard {
          modificationsFee
          waypointSymbol
        }
      }
      marketTrades {
        waypointSymbol
        symbol
        createdAt
        type
        tradeSymbolInfo {
          symbol
          requires {
            symbol
          }
          requiredBy {
            symbol
          }
        }
        marketTradeGood {
          symbol
          waypointSymbol
          type
          tradeVolume
          supply
          activity
          purchasePrice
          sellPrice
          createdAt
        }
      }
      constructionMaterials {
        waypointSymbol
        tradeSymbol
        required
        fulfilled
      }
      jumpGateConnections {
        from
        to
      }
      waypoints {
        symbol
        faction
        modifiers
        chartedBy
        chartedOn
        hasShipyard
        hasMarketplace
        x
        y
        lastScrap
        nextScrap
        waypointType
        traits
        isUnderConstruction
        orbitals
        orbits
        marketTrades {
          symbol
          type
          tradeSymbolInfo {
            symbol
            requires {
              symbol
            }
            requiredBy {
              symbol
            }
          }
          marketTradeGood {
            tradeVolume
            supply
            activity
            purchasePrice
            sellPrice
          }
        }
      }
      marketTransactions {
        trade_route_id
        mining_waypoint_symbol
        construction_shipment_id
        waypointSymbol
        shipSymbol
        tradeSymbol
        type
        units
        pricePerUnit
        totalPrice
        timestamp
        contract_id
      }
      contractDeliveries {
        contractId
        tradeSymbol
        destinationSymbol
        unitsRequired
        unitsFulfilled
        contract {
          id
          createdAt
          reservedFund
          factionSymbol
          contractType
          accepted
          onFulfilled
          deadline
          marketTransactionSummary {
            sum
            expenses
            income
            units
            purchaseUnits
            sellUnits
            purchaseTransactions
            sellTransactions
          }
        }
      }
      tradeRoutes {
        id
        reservedFund
        marketTransactionSummary {
          sum
          expenses
          income
          units
          purchaseUnits
          sellUnits
          purchaseTransactions
          sellTransactions
        }
        symbol
        shipSymbol
        PurchaseWaypointSymbol
        SellWaypointSymbol
        status
        tradeVolume
        predictedPurchasePrice
        predictedSellPrice
      }
      ships {
        symbol
        nav {
          waypointSymbol
          status
        }
        status {
          assignmentId
          fleetId
          tempAssignmentId
          tempFleetId
          status {
            __typename
          }
        }
      }
    }
  }
`);
