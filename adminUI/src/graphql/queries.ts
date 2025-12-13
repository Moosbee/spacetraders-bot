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
      }
    }
    fleetManager {
      busy
      channelState {
        usedCapacity
      }
    }
    tradeManager {
      busy
      channelState {
        usedCapacity
      }
    }
    miningManager {
      busy
      channelState {
        usedCapacity
      }
    }
    contractManager {
      busy
      channelState {
        usedCapacity
      }
    }
    scrappingManager {
      busy
      channelState {
        usedCapacity
      }
    }
    constructionManager {
      busy
      channelState {
        usedCapacity
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
