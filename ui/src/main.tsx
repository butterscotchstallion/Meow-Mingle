import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import { BrowserRouter } from "react-router-dom";
import { PrimeReactProvider } from "primereact/api";
import { client } from "./api/client.gen";
import { App } from "./App";
import { ErrorBoundary } from "./ErrorBoundary";
import "./index.css";

client.setConfig({
  baseUrl: "http://localhost:3000",
  credentials: "include",
});

client.interceptors.response.use((response, request) => {
  if (response.status === 401 && !request.url.includes("/auth/sign-in")) {
    window.location.href = "/signin";
  }
  return response;
});

createRoot(document.getElementById("root")!).render(
  <StrictMode>
    <ErrorBoundary>
      <PrimeReactProvider>
        <BrowserRouter>
          <App />
        </BrowserRouter>
      </PrimeReactProvider>
    </ErrorBoundary>
  </StrictMode>,
);
