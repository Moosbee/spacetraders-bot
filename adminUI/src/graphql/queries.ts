import { graphql } from "../gql/gql";

export const GET_MAIN_SITE_DATA = graphql(/* GraphQL */ `
  query GetMainSiteData {
    runInfo {
      resetDate
      nextResetDate
      version
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
        waypointType
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
        status {
          __typename
        }
      }
    }
  }
`);
