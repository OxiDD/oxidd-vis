import {FC} from "react";
import {ITabsHeaderProps} from "./props/ITabsHeaderProps";
import {ITabsContentProps} from "./props/ITabsContentProps";
import {IResizeHandleProps} from "./props/IResizeHandleProps";
import {IDropAreaProps} from "./props/IDropAreaProps";
import {ITabsContainerProps} from "./props/ITabsContainerProps";
import {IDragPreviewProps} from "./props/IDragPreviewProps";

/** The styled components to use for rendering the layout */
export type ILayoutComponents = {
    // Resizing rendering
    ResizeHandle: FC<IResizeHandleProps>;
    DropArea: FC<IDropAreaProps>;
    DragPreview: FC<IDragPreviewProps>;

    // Tab rendering
    TabsContainer: FC<ITabsContainerProps>;
    TabsHeader: FC<ITabsHeaderProps>;
    TabsContent: FC<ITabsContentProps>;
};
