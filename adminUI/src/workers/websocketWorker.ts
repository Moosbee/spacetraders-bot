import WsObject from "../models/WsObject";
import useMyStore, { backendUrl } from "../store";

const work = () => {
  console.log("Websocket worker started");

  let websocket: WebSocket | undefined;
  let reconnectTimeoutId: number | undefined;

  const setShip = useMyStore.getState().setShip;
  const setAgent = useMyStore.getState().setAgent;
  const setWebsocketConnected = useMyStore.getState().setWebsocketConnected;

  let wasConnected = false;

  const connect = () => {
    websocket = new WebSocket(`ws://${backendUrl}/ws/all`);

    websocket.onclose = () => {
      console.log("Disconnected from backend");

      if (wasConnected) {
        const title = `Disconnected from backend`;
        const notifaication = new Notification(title);
      }

      wasConnected = false;
      setWebsocketConnected(false);
      reconnectTimeoutId = setTimeout(connect, 1000);
    };

    websocket.onopen = () => {
      console.log("Connected to backend");
      wasConnected = true;
      setWebsocketConnected(true);
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
          setShip(wsObject.data.data);
          break;
        case "MyAgent":
          if (wsObject.data.data.account_id)
            setAgent({
              account_id: wsObject.data.data.account_id,
              symbol: wsObject.data.data.symbol,
              headquarters: wsObject.data.data.headquarters,
              credits: wsObject.data.data.credits,
              starting_faction: wsObject.data.data.starting_faction,
              ship_count: wsObject.data.data.ship_count,
              created_at: wsObject.data.data.created_at,
            });
          break;
        default:
          console.log(wsObject.data);
          break;
      }
    };
  };

  connect();
};

// work();
