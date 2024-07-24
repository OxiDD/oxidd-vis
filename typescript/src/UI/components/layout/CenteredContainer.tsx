import React, {FC} from "react";
import {ViewContainer} from "./ViewContainer";
import {css} from "@emotion/css";

/** A centered view container box */
export const CenteredContainer: FC<{maxWidth?: number}> = ({
    children,
    maxWidth = 800,
}) => (
    <ViewContainer className={css({display: "flex", justifyContent: "center"})}>
        <div style={{maxWidth, height: "fit-content"}}>{children}</div>
    </ViewContainer>
);
