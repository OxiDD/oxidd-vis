import ReactDOM from "react-dom";
import React from "react";
import {App} from "./App";
import {initializeIcons, ThemeProvider as FluentThemeProvider} from "@fluentui/react";
import {darkTheme} from "./theme";

initializeIcons();

ReactDOM.render(
    <FluentThemeProvider theme={darkTheme}>
        <App />
    </FluentThemeProvider>,
    document.getElementById("root")
);
