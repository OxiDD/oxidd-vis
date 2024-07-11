import ReactDOM from "react-dom";
import React from "react";
import {App} from "./App";
import {initializeIcons, ThemeProvider as FluentThemeProvider} from "@fluentui/react";
import {darkTheme} from "./theme";
import {installDevtools} from "./watchables/utils/devtools";

installDevtools();
initializeIcons();

ReactDOM.render(
    <FluentThemeProvider theme={darkTheme}>
        <App />
    </FluentThemeProvider>,
    document.getElementById("root")
);
