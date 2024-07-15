import React, {FC} from "react";
import {DefaultLayout} from "./layout/DefaultLayout";
import {LayoutState} from "./layout/LayoutState";
import {usePersistentMemo} from "./utils/usePersistentMemo";
import {Constant} from "./watchables/Constant";
import {AppState} from "./state/AppState";
import {all} from "./watchables/mutator/all";
import {chain} from "./watchables/mutator/chain";
import {Derived} from "./watchables/Derived";
import {Toggle} from "@fluentui/react";
import {useWatch} from "./watchables/react/useWatch";
import {LayoutWithClose} from "./UI/LayoutWithClose";
import {SettingsState} from "./state/SettingsState";
import {IViewComponent} from "./state/_types/IViewComponent";
import {Sidebar} from "./UI/SideBar";
import {TabContextMenu} from "./UI/TabContextMenu";
import {ThemeProvider as FluentThemeProvider} from "@fluentui/react";
import {darkTheme, lightTheme} from "./theme";

export const App: FC = () => {
    const app = usePersistentMemo(() => {
        const appState = new AppState();
        (window as any).app = appState;
        console.log(appState);

        const configuration = appState.configuration;
        configuration.loadProfilesData().commit();

        window.addEventListener("beforeunload", () =>
            appState.configuration.saveProfile().commit()
        );
        return appState;
    }, []);

    return (
        <ThemeProvider state={app}>
            <div style={{display: "flex", height: "100%"}}>
                <Sidebar state={app} projectUrl="https://google.com" />
                <div style={{flexGrow: 1, flexShrink: 1, minWidth: 0}}>
                    <UserLayout state={app} />
                </div>
            </div>
        </ThemeProvider>
    );
};

const UserLayout: FC<{state: AppState}> = ({state}) => {
    const watch = useWatch();
    return (
        <TabContextMenu state={state}>
            {onContext => (
                <LayoutWithClose
                    key={watch(state.configuration.profileID)} // Reinitialize when layout is switched
                    panelClosable={
                        // new Derived(watch =>
                        //     watch(app.settings.layout.deleteUnusedPanels) ? "never" : "always"
                        // )
                        new Constant("whenEmpty")
                    }
                    state={state.views.layoutState}
                    getContent={id => state.views.getPanelUI(id, Component, onContext)}
                />
            )}
        </TabContextMenu>
    );
};

const Component: IViewComponent = ({view}) => {
    if (view instanceof SettingsState) return <SettingsView settings={view} />;

    return <div>Not found</div>;
};

const ThemeProvider: FC<{state: AppState}> = ({state, children}) => {
    const watch = useWatch();
    return (
        <FluentThemeProvider
            theme={watch(state.settings.global).darkMode ? darkTheme : lightTheme}>
            {children}
        </FluentThemeProvider>
    );
};

const SettingsView: FC<{settings: SettingsState}> = ({settings}) => {
    const watch = useWatch();
    return (
        <div>
            Delete unused panels:
            <Toggle
                checked={watch(settings.layout.deleteUnusedPanels)}
                onChange={(_, checked) =>
                    settings.layout.deleteUnusedPanels.set(checked ?? false).commit()
                }
            />
        </div>
    );
};
