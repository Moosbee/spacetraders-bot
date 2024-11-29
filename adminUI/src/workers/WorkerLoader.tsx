import { useEffect } from "react";
import websocketWorker from "./websocketWorker?sharedworker";

function WorkerLoader() {
  useEffect(() => {
    // Create a new web worker
    const wsWorker = new websocketWorker();
    // Clean up the worker when the component unmounts
    return () => {
      wsWorker.port.close();
    };
  }, []);
  return null;
}

export default WorkerLoader;
