import { graphql } from "../gql/gql";

export const REPOPULATE_SYSTEMS_WITH_FLEETS_FROM_JUMP_GATE = graphql(
  /* GraphQL */ `
    mutation RepopulateSystemsWithFleetsFromJumpGate($jumpGate: String!) {
      repopulateSystemsWithFleetsFromJumpGate(jumpGate: $jumpGate)
    }
  `,
);

export const REPOPULATE_SYSTEMS_WITH_FLEETS_FROM_SYSTEM = graphql(
  /* GraphQL */ `
    mutation RepopulateSystemsWithFleetsFromSystem($systemSymbol: String!) {
      repopulateSystemWithFleets(system: $systemSymbol)
    }
  `,
);

export const REGENERATE_FLEET_ASSIGNMENTS = graphql(/* GraphQL */ `
  mutation RegenerateFleetAssignments($fleet_id: Int!) {
    regenerateFleetAssignments(by: { fleet: $fleet_id })
  }
`);

export const EDIT_FLEET = graphql(/* GraphQL */ `
  mutation EditFleet($id: Int!, $config: InputFleetConfig!) {
    editFleet(id: $id, config: $config) {
      id
    }
  }
`);

export const FORCE_ASSIGN_SHIP = graphql(/* GraphQL */ `
  mutation ForceAssignShip(
    $shipSymbol: String!
    $assignmentId: Int!
    $temp: Boolean!
  ) {
    forceAssignShip(
      shipSymbol: $shipSymbol
      assignmentId: $assignmentId
      temp: $temp
    ) {
      symbol
      assignmentId
      tempAssignmentId
    }
  }
`);

export const SHUTDOWN = graphql(/* GraphQL */ `
  mutation Shutdown(
    $global: Boolean!
    $run: Boolean!
    $slowManagers: Boolean!
    $fastManagers: Boolean!
    $slowShips: Boolean!
    $fastShips: Boolean!
  ) {
    shutdown(
      global: $global
      run: $run
      slowManagers: $slowManagers
      fastManagers: $fastManagers
      slowShips: $slowShips
      fastShips: $fastShips
    ) {
      globalCancelToken
      runCancelToken
      fastManagerCancelToken
      slowManagerCancelToken
      fastShipCancelToken
      slowShipCancelToken
    }
  }
`);
