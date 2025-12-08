/* eslint-disable */
import * as types from './graphql';
import { TypedDocumentNode as DocumentNode } from '@graphql-typed-document-node/core';

/**
 * Map of all GraphQL operations in the project.
 *
 * This map has several performance disadvantages:
 * 1. It is not tree-shakeable, so it will include all operations in the project.
 * 2. It is not minifiable, so the string of a GraphQL query will be multiple times inside the bundle.
 * 3. It does not support dead code elimination, so it will add unused operations.
 *
 * Therefore it is highly recommended to use the babel or swc plugin for production.
 * Learn more about it here: https://the-guild.dev/graphql/codegen/plugins/presets/preset-client#reducing-bundle-size
 */
type Documents = {
    "\n  query GetMainSiteData {\n    runInfo {\n      resetDate\n      nextResetDate\n      version\n      agent {\n        symbol\n        credits\n        shipCount\n      }\n      headquartersSystem {\n        symbol\n        constructionMaterials {\n          waypointSymbol\n          tradeSymbol\n          required\n          fulfilled\n        }\n      }\n    }\n    systems(onlyWithFleetsOrShips: true) {\n      symbol\n      waypoints {\n        symbol\n        waypointType\n        chartedBy\n        hasMarketplace\n        hasShipyard\n      }\n    }\n    fleets {\n      id\n      systemSymbol\n      fleetType\n      active\n      assignments {\n        id\n        priority\n        rangeMin\n        cargoMin\n        ship {\n          symbol\n        }\n      }\n    }\n    shipAssignments(by: { open: true }) {\n      id\n      fleetId\n      fleet {\n        systemSymbol\n        fleetType\n      }\n    }\n    ships {\n      symbol\n      registrationRole\n      status {\n        assignmentId\n        status {\n          __typename\n        }\n      }\n    }\n  }\n": typeof types.GetMainSiteDataDocument,
};
const documents: Documents = {
    "\n  query GetMainSiteData {\n    runInfo {\n      resetDate\n      nextResetDate\n      version\n      agent {\n        symbol\n        credits\n        shipCount\n      }\n      headquartersSystem {\n        symbol\n        constructionMaterials {\n          waypointSymbol\n          tradeSymbol\n          required\n          fulfilled\n        }\n      }\n    }\n    systems(onlyWithFleetsOrShips: true) {\n      symbol\n      waypoints {\n        symbol\n        waypointType\n        chartedBy\n        hasMarketplace\n        hasShipyard\n      }\n    }\n    fleets {\n      id\n      systemSymbol\n      fleetType\n      active\n      assignments {\n        id\n        priority\n        rangeMin\n        cargoMin\n        ship {\n          symbol\n        }\n      }\n    }\n    shipAssignments(by: { open: true }) {\n      id\n      fleetId\n      fleet {\n        systemSymbol\n        fleetType\n      }\n    }\n    ships {\n      symbol\n      registrationRole\n      status {\n        assignmentId\n        status {\n          __typename\n        }\n      }\n    }\n  }\n": types.GetMainSiteDataDocument,
};

/**
 * The graphql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 *
 *
 * @example
 * ```ts
 * const query = graphql(`query GetUser($id: ID!) { user(id: $id) { name } }`);
 * ```
 *
 * The query argument is unknown!
 * Please regenerate the types.
 */
export function graphql(source: string): unknown;

/**
 * The graphql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function graphql(source: "\n  query GetMainSiteData {\n    runInfo {\n      resetDate\n      nextResetDate\n      version\n      agent {\n        symbol\n        credits\n        shipCount\n      }\n      headquartersSystem {\n        symbol\n        constructionMaterials {\n          waypointSymbol\n          tradeSymbol\n          required\n          fulfilled\n        }\n      }\n    }\n    systems(onlyWithFleetsOrShips: true) {\n      symbol\n      waypoints {\n        symbol\n        waypointType\n        chartedBy\n        hasMarketplace\n        hasShipyard\n      }\n    }\n    fleets {\n      id\n      systemSymbol\n      fleetType\n      active\n      assignments {\n        id\n        priority\n        rangeMin\n        cargoMin\n        ship {\n          symbol\n        }\n      }\n    }\n    shipAssignments(by: { open: true }) {\n      id\n      fleetId\n      fleet {\n        systemSymbol\n        fleetType\n      }\n    }\n    ships {\n      symbol\n      registrationRole\n      status {\n        assignmentId\n        status {\n          __typename\n        }\n      }\n    }\n  }\n"): (typeof documents)["\n  query GetMainSiteData {\n    runInfo {\n      resetDate\n      nextResetDate\n      version\n      agent {\n        symbol\n        credits\n        shipCount\n      }\n      headquartersSystem {\n        symbol\n        constructionMaterials {\n          waypointSymbol\n          tradeSymbol\n          required\n          fulfilled\n        }\n      }\n    }\n    systems(onlyWithFleetsOrShips: true) {\n      symbol\n      waypoints {\n        symbol\n        waypointType\n        chartedBy\n        hasMarketplace\n        hasShipyard\n      }\n    }\n    fleets {\n      id\n      systemSymbol\n      fleetType\n      active\n      assignments {\n        id\n        priority\n        rangeMin\n        cargoMin\n        ship {\n          symbol\n        }\n      }\n    }\n    shipAssignments(by: { open: true }) {\n      id\n      fleetId\n      fleet {\n        systemSymbol\n        fleetType\n      }\n    }\n    ships {\n      symbol\n      registrationRole\n      status {\n        assignmentId\n        status {\n          __typename\n        }\n      }\n    }\n  }\n"];

export function graphql(source: string) {
  return (documents as any)[source] ?? {};
}

export type DocumentType<TDocumentNode extends DocumentNode<any, any>> = TDocumentNode extends DocumentNode<  infer TType,  any>  ? TType  : never;