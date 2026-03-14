import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import { BrowserRouter } from "react-router-dom";
import { PrimeReactProvider } from "primereact/api";
import { client } from "./api/client.gen";
import { App } from "./App";
import "./index.css";

client.setConfig({
  baseUrl: "http://localhost:3000",
  credentials: "include",
});

createRoot(document.getElementById("root")!).render(
  <StrictMode>
    <PrimeReactProvider>
      <BrowserRouter>
        <App />
      </BrowserRouter>
    </PrimeReactProvider>
  </StrictMode>,
);
