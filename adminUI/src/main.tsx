import { ApolloProvider } from "@apollo/client/react";
import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import { Provider } from "react-redux";
import { BrowserRouter } from "react-router-dom";
import { PersistGate } from "redux-persist/integration/react";
import client from "./apolloClient.ts";
import "./index.css";
import MyApp from "./MyApp.tsx";
import { persistor, store } from "./redux/store.ts";

const promise = await Notification.requestPermission();

console.log("Notification permission:", promise);

createRoot(document.getElementById("root")!).render(
  <StrictMode>
    <ApolloProvider client={client}>
      <Provider store={store}>
        <PersistGate loading={null} persistor={persistor}>
          <BrowserRouter basename={import.meta.env.BASE_URL}>
            <MyApp />
          </BrowserRouter>
        </PersistGate>
      </Provider>
    </ApolloProvider>
  </StrictMode>
);
