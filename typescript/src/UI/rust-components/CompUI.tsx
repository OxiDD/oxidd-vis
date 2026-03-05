import React from "react";
import {ICompUI} from "./_types/ICompUI";
import {CompositeCompUI} from "./other/CompositeCompUI";
import {ButtonCompUI} from "./other/ButtonCompUI";
import {MessageBar, MessageBarType} from "@fluentui/react";
import {LabelCompUI} from "./other/LabelCompUI";
import {StringInputCompUI} from "./inputs/StringInputCompUI";
import {when} from "../../utils/when";
import {DynCompUI} from "./other/DynCompUI";
import {TextCompUI} from "./other/TextCompUI";
import {PanelHandleCompUI} from "./other/PanelHandleCompUI";
import {TooltipCompUI} from "./other/TooltipCompUI";
import {OverlayCompUI} from "./other/OverlayCompUI";
import {FillCompUI} from "./other/FillCompUI";
import {CompositeItemCompUI} from "./other/CompositeItemCompUI";
import {ContainerCompUI} from "./other/ContainerCompUI";

export const CompUI: ICompUI = ({data: d, className, aria}) => {
    const p = {className, aria, ChildComp: CompUI};
    return (
        when(d.as_composite(), r => <CompositeCompUI data={r} {...p} />) ??
        when(d.as_container(), r => <ContainerCompUI data={r} {...p} />) ??
        when(d.as_label(), r => <LabelCompUI data={r} {...p} />) ??
        when(d.as_string_input(), r => <StringInputCompUI data={r} {...p} />) ??
        when(d.as_button(), r => <ButtonCompUI data={r} {...p} />) ??
        when(d.as_dyn(), r => <DynCompUI data={r} {...p} />) ??
        when(d.as_text(), r => <TextCompUI data={r} {...p} />) ??
        when(d.as_panel_handle(), r => <PanelHandleCompUI data={r} {...p} />) ??
        when(d.as_tooltip(), r => <TooltipCompUI data={r} {...p} />) ??
        when(d.as_overlay(), r => <OverlayCompUI data={r} {...p} />) ??
        when(d.as_fill(), r => <FillCompUI data={r} {...p} />) ??
        when(d.as_composite_item(), r => <CompositeItemCompUI data={r} {...p} />) ??
        when(d.as_panel(), r => <></>) ??
        componentNotFound
    );
};

const componentNotFound = (
    <MessageBar messageBarType={MessageBarType.error} isMultiline={false}>
        Component not found
    </MessageBar>
);
