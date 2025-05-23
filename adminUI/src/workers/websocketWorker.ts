import { backendUrl } from "../data";
import WsObject from "../models/WsObject";
import { setMyAgent } from "../redux/slices/agentSlice";
import {
  selectConnectWebsocket,
  setWebsocketConnected,
} from "../redux/slices/configSlice";
import { setShip } from "../redux/slices/shipSlice";
import { store } from "../redux/store";

console.log("Websocket worker started");

// store.subscribe(() => {
//   const state = store.getState();
//   console.log("Websocket worker state", state);
// });

const work = () => {
  console.log("Websocket worker working started");

  let websocket: WebSocket | undefined;
  let reconnectTimeoutId: number | undefined;

  let wasConnected = false;
  let is_connected = false;

  store.subscribe(() => {
    const shouldConnect = selectConnectWebsocket(store.getState());
    if (is_connected && !shouldConnect && wasConnected && websocket) {
      console.log("Disconnecting from backend");
      is_connected = false;
      websocket.close();
    }
  });

  const connect = () => {
    const shouldConnect = selectConnectWebsocket(store.getState());

    if (!shouldConnect) {
      console.log("Not connecting");
      reconnectTimeoutId = setTimeout(connect, 1000);
      return;
    }

    console.log("Connecting to backend");

    websocket = new WebSocket(`ws://${backendUrl}/ws/all`);

    websocket.onclose = () => {
      console.log("Disconnected from backend");
      is_connected = false;

      if (wasConnected) {
        const title = `Disconnected from backend`;
        const notification = new Notification(title);

        notification.onclick = () => {
          console.log("Notification clicked");
        };
      }

      wasConnected = false;
      store.dispatch(setWebsocketConnected(false));
      reconnectTimeoutId = setTimeout(connect, 1000);
    };

    websocket.onopen = () => {
      console.log("Connected to backend");
      wasConnected = true;
      is_connected = true;
      store.dispatch(setWebsocketConnected(true));

      if (reconnectTimeoutId !== undefined) {
        clearTimeout(reconnectTimeoutId);
        reconnectTimeoutId = undefined;
      }
    };

    websocket.onmessage = (event) => {
      // console.log(event.data);
      const wsObject: WsObject = JSON.parse(event.data);
      // console.log("WS OBJECT", Date.now(), wsObject);
      switch (wsObject.data.type) {
        case "RustShip":
          store.dispatch(setShip(wsObject.data.data));

          break;
        case "MyAgent":
          if (wsObject.data.data.account_id)
            store.dispatch(
              setMyAgent({
                account_id: wsObject.data.data.account_id,
                symbol: wsObject.data.data.symbol,
                headquarters: wsObject.data.data.headquarters,
                credits: wsObject.data.data.credits,
                starting_faction: wsObject.data.data.starting_faction,
                ship_count: wsObject.data.data.ship_count,
                created_at: wsObject.data.data.created_at,
              })
            );
          break;
        default:
          console.log(wsObject.data);
          break;
      }
    };
  };
  connect();
};

work();
