import {Icon, Link, useTheme} from "@fluentui/react";
import React, {FC} from "react";
import {ViewState} from "../../../state/views/ViewState";
import {AppState} from "../../../state/AppState";
import {css} from "@emotion/css";
import {ViewContainer} from "../../components/ViewContainer";
import {CenteredContainer} from "../../components/CenteredContainer";

export const Info: FC<{app: AppState}> = ({app}) => {
    const theme = useTheme();
    const link = (text: string, openView: ViewState) => (
        <Link onClick={() => app.open(openView).commit()}>{text}</Link>
    );

    return (
        <CenteredContainer>
            <h1 style={{color: theme.palette.themePrimary, fontSize: 40, marginTop: 10}}>
                <Icon
                    iconName="GitGraph"
                    styles={{
                        root: {marginRight: 10, fontSize: 45, verticalAlign: "bottom"},
                    }}
                />
                BDD-viz
            </h1>
            <p>
                BDD-viz (temporary name) is a Binary Decision Diagram (BDD) visualization
                tool. It is integrated with OxiDD to allow real world BDDs to be
                visualized. It additionally supports various other decision diagram types,
                and can be used for experimentation.
            </p>
            <p>Settings: {link("settings", app.settings)}</p>
        </CenteredContainer>
    );
};
