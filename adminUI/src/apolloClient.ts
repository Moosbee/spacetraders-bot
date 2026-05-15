import { ApolloClient, HttpLink, InMemoryCache, split } from "@apollo/client";
import { GraphQLWsLink } from "@apollo/client/link/subscriptions";
import { getMainDefinition } from "@apollo/client/utilities";
import { createClient } from "graphql-ws";

import { backendUrl } from "./data";

const httpLink = new HttpLink({ uri: "http://127.0.0.1:8780/" });

const wsLink =
  typeof window === "undefined"
    ? null
    : new GraphQLWsLink(
        createClient({
          url: `ws://${backendUrl}/ws`,
        }),
      );

const link =
  wsLink === null
    ? httpLink
    : split(
        ({ query }) => {
          const definition = getMainDefinition(query);
          return (
            definition.kind === "OperationDefinition" &&
            definition.operation === "subscription"
          );
        },
        wsLink,
        httpLink,
      );

const client = new ApolloClient({
  link,
  cache: new InMemoryCache(), // InMemoryCache is useful for caching queries
});

export default client;
