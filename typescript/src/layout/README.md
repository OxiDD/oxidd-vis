# Layout

This folder contains a layout management system. It includes the system that allows panels to be repositioned, but not the contents of the panels.
There are a couple of dependencies to utilities outside this directory, but it's mostly self-contained. I would like to release this as a standalone package eventually, in order to use this same system in other projects.

## Usage

Sample code:

```tsx
import React, {FC} from "react";
import {DefaultLayout} from "./layout/DefaultLayout";
import {LayoutState} from "./layout/LayoutState";
import {usePersistentMemo} from "./utils/usePersistentMemo";
import {Constant} from "./watchables/Constant";

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
                    name: "tab " + id,
                    id,
                    content: <div>hoi {id}</div>,
                });
            }}
        />
    );
};
```

You are responsible for providing sensible content, and creating tabs.
This can be done by maintaining a mapping from tab ids to content, and adding the appropriate content when creating a new tab id.
