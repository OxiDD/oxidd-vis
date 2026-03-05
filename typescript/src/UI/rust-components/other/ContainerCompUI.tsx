import React from "react";
import {ContainerComp, UIBackgroundColor} from "oxidd-vis-rust";
import {NFC} from "../../../utils/_types/NFC";
import {ICompUI} from "../_types/ICompUI";
import {IAriaRef} from "../_types/IAriaRef";
import {Theme, useTheme} from "@fluentui/react";
import {useWatch} from "../../../watchables/react/useWatch";
import {css} from "@emotion/css";
import {multiplySize} from "../../../utils/multiplySize";

export const ContainerCompUI: NFC<{
    data: ContainerComp;
    ChildComp: ICompUI;
    className?: string;
    aria?: IAriaRef;
}> = ({data, ChildComp, className, aria}) => {
    const watch = useWatch();
    const theme = useTheme();
    const spacing = theme.spacing.m;
    const child = watch(data.content);
    const marginLeft = watch(data.margin_left);
    const marginRight = watch(data.margin_right);
    const marginTop = watch(data.margin_top);
    const marginBottom = watch(data.margin_bottom);
    const paddingLeft = watch(data.padding_left);
    const paddingRight = watch(data.padding_right);
    const paddingTop = watch(data.padding_top);
    const paddingBottom = watch(data.padding_bottom);
    const width = watch(data.min_width);
    const height = watch(data.min_height);
    const color = watch(data.background_color);

    return (
        <div
            className={`${className} ${css({
                position: "relative",
                boxSizing: "border-box",
                backgroundColor: color ? getColor(theme, color) : undefined,
                width,
                height,
                marginLeft: multiplySize(marginLeft, spacing),
                marginRight: multiplySize(marginRight, spacing),
                marginTop: multiplySize(marginTop, spacing),
                marginBottom: multiplySize(marginBottom, spacing),
                paddingLeft: multiplySize(paddingLeft, spacing),
                paddingRight: multiplySize(paddingRight, spacing),
                paddingTop: multiplySize(paddingTop, spacing),
                paddingBottom: multiplySize(paddingBottom, spacing),
            })}`}>
            <ChildComp data={child} aria={aria} />
        </div>
    );
};

function getColor(theme: Theme, color: UIBackgroundColor): string {
    const c = theme.palette;
    switch (color) {
        case UIBackgroundColor.NeutralLight:
            return c.neutralLight;
        case UIBackgroundColor.NeutralMid:
            return c.neutralPrimary;
        case UIBackgroundColor.NeutralDark:
            return c.neutralDark;
        case UIBackgroundColor.HighlightPrimary:
            return c.themePrimary;
        case UIBackgroundColor.HighlightSecondary:
            return c.themeSecondary;
    }
}
