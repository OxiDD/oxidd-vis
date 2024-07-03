import React, {FC} from "react";
import {IResizeHandleProps} from "../_types/props/IResizeHandleProps";
import {useTheme} from "@fluentui/react";

export const ResizeHandle: FC<IResizeHandleProps> = ({direction}) => {
    const theme = useTheme();
    return (
        <div
            style={{
                width: direction == "horizontal" ? 10 : "100%",
                height: direction == "vertical" ? 10 : "100%",
                boxShadow: "inset #0000004d 0px 0px 6px 2px",
            }}></div>
    );
};
