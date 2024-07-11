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

export const App: FC = () => {
    const app = usePersistentMemo(() => {
        const appState = new AppState();

        chain(push => {
            for (let i = 0; i < 5; i++) {
                const dummy = new DummyViewState();
                push(dummy.name.set("stuff " + i));
                push(appState.views.show(dummy));
            }
        }).commit();
        return appState;
    }, []);

    return (
        <DefaultLayout
            state={app.views.layoutState}
            getContent={id => app.views.getPanelUI(id, components)}
        />
    );
};

const components: IViewComponents = {
    none: () => <div>Not found</div>,
    dummy: () => <div>Dummy</div>,
};
