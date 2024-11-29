import RustShip from "../models/ship";
import useMyStore, { backendUrl } from "../store";

const work = () => {
  console.log("notificationWorker");

  const websocket = new WebSocket(`ws://${backendUrl}/ws/ships`);

  websocket.onclose = () => {
    console.log("Disconnected from backend");
  };

  const setShip = useMyStore.getState().setShip;

  websocket.onopen = () => {
    console.log("Connected to backend");
  };

  websocket.onmessage = (event) => {
    console.log(event.data);
    const rsShip: RustShip = JSON.parse(event.data);
    console.log(rsShip);
    setShip(rsShip);
  };
};

work();
