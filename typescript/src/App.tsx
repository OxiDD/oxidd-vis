import React, {FC} from "react";
import {DefaultLayout} from "./layout/DefaultLayout";
import {LayoutState} from "./layout/LayoutState";
import {usePersistentMemo} from "./utils/usePersistentMemo";
import {Constant} from "./watchables/Constant";
import {VizTest} from "./VizTest";

export const App: FC = () => {
    const layoutState = usePersistentMemo(() => {
        const layout = new LayoutState();
        layout
            .openTab("0", "0", () => console.log("closed 0"))
            .chain(layout.openTab("0", "1", () => console.log("closed 1")))
            .chain(layout.openTab("0", "2", () => console.log("closed 2")))
            .chain(layout.openTab("0", "3", () => console.log("closed 3")))
            .commit();
        return layout;
    }, []);
    return (
        <DefaultLayout
            state={layoutState}
            getContent={id => {
                return new Constant({
                    name: "hoi " + id,
                    id,
                    content: <VizTest />,
                });
            }}
        />
    );
};
