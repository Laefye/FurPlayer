import * as React from "react";
import * as ReactDOM from "react-dom/client";
import './index.css';
import App from "./App";
import { EngineContext } from "./Engine";

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <EngineContext>
      <App />
    </EngineContext>
  </React.StrictMode>,
);