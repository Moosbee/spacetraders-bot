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
