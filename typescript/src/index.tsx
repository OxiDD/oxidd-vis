import ReactDOM from "react-dom";
import React from "react";
import {App} from "./App";
import {initializeIcons, ThemeProvider as FluentThemeProvider} from "@fluentui/react";
import {darkTheme} from "./theme";
import {installDevtools} from "./watchables/utils/devtools";

installDevtools();
initializeIcons();

Error.stackTraceLimit = 30;
ReactDOM.render(<App />, document.getElementById("root"));
