import { ApolloClient, HttpLink, InMemoryCache } from "@apollo/client";

const httpLink = new HttpLink({ uri: "http://127.0.0.1:8780/" });

const client = new ApolloClient({
  link: httpLink,
  cache: new InMemoryCache(), // InMemoryCache is useful for caching queries
});

export default client;
