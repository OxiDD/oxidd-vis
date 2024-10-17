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
import {CustomLayout} from "./UI/components/layout/CustomLayout";
import {SettingsState} from "./state/SettingsState";
import {IViewComponent} from "./state/_types/IViewComponent";
import {Sidebar} from "./UI/SideBar";
import {TabContextMenu} from "./UI/TabContextMenu";
import {ThemeProvider as FluentThemeProvider} from "@fluentui/react";
import {darkTheme, lightTheme} from "./theme";
import {Settings} from "./UI/views/settings/Settings";
import {Info} from "./UI/views/info/Info";
import {ViewContainer} from "./UI/components/layout/ViewContainer";
import {DiagramCollectionState} from "./state/diagrams/DiagramCollectionState";
import {DiagramCollection} from "./UI/views/diagramCollection/DiagramCollection";
import {ThemeProvider} from "./UI/providers/ThemeProvider";
import {DiagramVisualizationState} from "./state/diagrams/DiagramVisualizationState";
import {DiagramVisualization} from "./UI/views/diagramVisualization/DiagramVisualization";
import {ViewManagerProvider} from "./UI/providers/ViewManagerContext";
import {ToolbarProvider} from "./UI/providers/ToolbarContext";
import {ToolbarState} from "./state/toolbar/ToolbarState";
import {Toolbar} from "./UI/views/toolbar/Toolbar";
import {CenteredContainer} from "./UI/components/layout/CenteredContainer";
import {HttpDiagramCollectionTargetState} from "./state/diagrams/collections/HttpDiagramCollectionState";
import {DiagramCollectionTarget} from "./UI/views/diagramCollection/types/util/DiagramCollectionTarget";

export const App: FC = () => {
    const app = usePersistentMemo(() => {
        const appState = new AppState();
        (window as any).app = appState;
        console.log(appState);

        const configuration = appState.configuration;
        const loaded = configuration.loadProfilesData().commit();
        if (!loaded) {
            // Open the info screen on first use
            appState.views.open(appState).commit();
        }

        window.addEventListener("beforeunload", () => {
            appState.configuration.saveProfile().commit();
        });
        return appState;
    }, []);

    return (
        <ThemeProvider state={app}>
            <ViewManagerProvider value={app.views}>
                <ToolbarProvider value={app.toolbar}>
                    <div style={{display: "flex", height: "100%"}}>
                        <Sidebar state={app} projectUrl="https://google.com" />
                        <div style={{flexGrow: 1, flexShrink: 1, minWidth: 0}}>
                            <UserLayout state={app} />
                        </div>
                    </div>
                </ToolbarProvider>
            </ViewManagerProvider>
        </ThemeProvider>
    );
};

const Component: IViewComponent = ({view}) => {
    if (view instanceof SettingsState) return <Settings settings={view} />;
    if (view instanceof AppState) return <Info app={view} />;
    if (view instanceof DiagramCollectionState)
        return (
            <CenteredContainer>
                <DiagramCollection collection={view.collection} />
            </CenteredContainer>
        );
    if (view instanceof HttpDiagramCollectionTargetState)
        return <DiagramCollectionTarget target={view} />;
    if (view instanceof DiagramVisualizationState)
        return <DiagramVisualization visualization={view} />;
    if (view instanceof ToolbarState) return <Toolbar toolbar={view} />;

    return <ViewContainer>Not found</ViewContainer>;
};

const UserLayout: FC<{state: AppState}> = ({state}) => {
    const watch = useWatch();
    return (
        <TabContextMenu>
            {onContext => (
                <CustomLayout
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
