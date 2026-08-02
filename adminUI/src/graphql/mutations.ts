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
