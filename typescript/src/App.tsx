import React, {FC} from "react";
import {DefaultLayout} from "./layout/DefaultLayout";
import {LayoutState} from "./layout/LayoutState";
import {usePersistentMemo} from "./utils/usePersistentMemo";
import {Constant} from "./watchables/Constant";
import {VizTest} from "./VizTest";
import {AppState} from "./state/AppState";
import {IViewComponents} from "./state/_types/IViewComponents";
import {DummyViewState} from "./state/views/types/DummyViewState";
import {all} from "./watchables/mutator/all";
import {chain} from "./watchables/mutator/chain";
import {Derived} from "./watchables/Derived";
import {Toggle} from "@fluentui/react";
import {useWatch} from "./watchables/react/useWatch";
import {LayoutWithClose} from "./LayoutWithClose";

export const App: FC = () => {
    const app = usePersistentMemo(() => {
        const appState = new AppState();
        (window as any).app = appState;
        console.log(appState);
        const configuration = appState.configuration;

        const init = (loaded: boolean) =>
            chain(push => {
                if (loaded) return;
                for (let i = 0; i < 5; i++) {
                    const dummy = new DummyViewState(appState.settings);
                    push(dummy.name.set("stuff " + i));
                    push(appState.views.show(dummy));
                }
            });
        configuration
            .loadProfilesData()
            .chain(s => appState.initSpecialViews().map(() => s))
            .chain(init)
            .commit();
        // init(false).commit();

        window.addEventListener("beforeunload", () =>
            appState.configuration.saveProfile().commit()
        );
        return appState;
    }, []);

    return (
        <LayoutWithClose
            panelClosable={
                // new Derived(watch =>
                //     watch(app.settings.layout.deleteUnusedPanels) ? "never" : "always"
                // )
                new Constant("whenEmpty")
            }
            state={app.views.layoutState}
            getContent={id => app.views.getPanelUI(id, components)}
        />
    );
};

const components: IViewComponents = {
    none: () => <div>Not found</div>,
    dummy: ({view}: {view: DummyViewState}) => {
        const watch = useWatch();
        return (
            <div>
                Dummy
                <Toggle
                    checked={watch(view.settings.layout.deleteUnusedPanels)}
                    onChange={(_, checked) =>
                        view.settings.layout.deleteUnusedPanels
                            .set(checked ?? false)
                            .commit()
                    }
                />
            </div>
        );
    },
};
