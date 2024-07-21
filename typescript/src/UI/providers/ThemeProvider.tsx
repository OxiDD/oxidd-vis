import React, {FC} from "react";
import {AppState} from "../../state/AppState";
import {useWatch} from "../../watchables/react/useWatch";
import {ThemeProvider as FluentThemeProvider} from "@fluentui/react";
import {darkTheme, lightTheme} from "../../theme";

export const ThemeProvider: FC<{state: AppState}> = ({state, children}) => {
    const watch = useWatch();
    return (
        <FluentThemeProvider
            theme={watch(state.settings.global).darkMode ? darkTheme : lightTheme}>
            {children}
        </FluentThemeProvider>
    );
};
