import React, {FC} from "react";
import {ILayoutProps} from "./_types/ILayoutProps";
import {LayoutPanel} from "./LayoutPanel";
import {TabsRenderer} from "./TabsRenderer";
import {Watcher} from "../watchables/react/Watcher";

/**
 * The layout entry component, where styling components still have to be provided
 */
export const Layout: FC<ILayoutProps> = ({state, components, getContent}) => (
    <div className="layout-root" style={{height: "100%"}}>
        <Watcher>
            {watch => (
                <LayoutPanel
                    state={state}
                    components={components}
                    panel={watch(state.layoutState)}
                    getContent={getContent}
                />
            )}
        </Watcher>
        <Watcher>
            {watch => (
                <components.DragPreview data={watch(state.draggingData)} state={state} />
            )}
        </Watcher>
        <TabsRenderer state={state} getContent={getContent} />
    </div>
);
