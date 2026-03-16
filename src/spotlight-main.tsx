import React from "react";
import ReactDOM from "react-dom/client";
import { SpotlightApp } from "./SpotlightApp";
import "./index.css";

ReactDOM.createRoot(document.getElementById("spotlight-root")!).render(
    <React.StrictMode>
        <SpotlightApp />
    </React.StrictMode>
);
