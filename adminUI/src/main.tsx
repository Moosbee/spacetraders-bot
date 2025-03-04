import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import { BrowserRouter } from "react-router-dom";
import "./index.css";
import MyApp from "./MyApp.tsx";

// eslint-disable-next-line @typescript-eslint/no-unused-vars
const _promise = await Notification.requestPermission();

createRoot(document.getElementById("root")!).render(
  <StrictMode>
    <BrowserRouter basename={import.meta.env.BASE_URL}>
      <MyApp />
    </BrowserRouter>
  </StrictMode>
);
