import WsObject from "../models/WsObject";
import useMyStore, { backendUrl } from "../store";

const work = () => {
  console.log("Websocket worker started");

  let websocket: WebSocket | undefined;
  let reconnectTimeoutId: number | undefined;

  const setShip = useMyStore.getState().setShip;
  const setAgent = useMyStore.getState().setAgent;
  const setWebsocketConnected = useMyStore.getState().setWebsocketConnected;

  const connect = () => {
    websocket = new WebSocket(`ws://${backendUrl}/ws/all`);

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
      const wsObject: WsObject = JSON.parse(event.data);
      console.log(wsObject);
      switch (wsObject.data.type) {
        case "RustShip":
          setShip(wsObject.data.data);
          break;
        case "MyAgent":
          setAgent(wsObject.data.data);
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
