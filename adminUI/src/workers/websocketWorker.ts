import RustShip from "../models/ship";
import useMyStore, { backendUrl } from "../store";

const work = () => {
  console.log("Websocket worker started");

  let websocket: WebSocket | undefined;
  let reconnectTimeoutId: number | undefined;

  const setShip = useMyStore.getState().setShip;
  const setWebsocketConnected = useMyStore.getState().setWebsocketConnected;

  const connect = () => {
    websocket = new WebSocket(`ws://${backendUrl}/ws/ships`);

    websocket.onclose = () => {
      console.log("Disconnected from backend");
      setWebsocketConnected(false);
      reconnectTimeoutId = setTimeout(connect, 1000);
    };

    websocket.onopen = () => {
      console.log("Connected to backend");
      setWebsocketConnected(true);
      if (reconnectTimeoutId !== undefined) {
        clearTimeout(reconnectTimeoutId);
        reconnectTimeoutId = undefined;
      }
    };

    websocket.onmessage = (event) => {
      console.log(event.data);
      const rsShip: RustShip = JSON.parse(event.data);
      console.log(rsShip);
      setShip(rsShip);
    };
  };

  connect();
};

work();
