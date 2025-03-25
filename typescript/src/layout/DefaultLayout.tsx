import React, {FC} from "react";
import {ILayoutProps} from "./_types/ILayoutProps";
import {ILayoutComponents} from "./_types/ILayourComponents";
import {Layout} from "./Layout";
import {DropArea} from "./styledComponents/DropArea";
import {ResizeHandle} from "./styledComponents/ResizeHandle";
import {TabsContent} from "./styledComponents/TabsContent";
import {TabsHeader} from "./styledComponents/Tabsheader";
import {TabsContainer} from "./styledComponents/TabsContainer";
import {DragPreview} from "./styledComponents/DragPreview";

/**
 * The layout entry component, where styling components are already provided
 */
export const DefaultLayout: FC<
    Omit<ILayoutProps, "components"> & {components?: Partial<ILayoutComponents>}
> = ({state, components = {}, getContent}) => {
    return (
        <Layout
            state={state}
            getContent={getContent}
            components={{
                DropArea,
                DragPreview,
                ResizeHandle,
                TabsContainer,
                TabsContent,
                TabsHeader,
                ...components,
            }}
        />
    );
};
