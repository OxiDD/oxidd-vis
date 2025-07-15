import React, {FC, useCallback, useEffect, useMemo} from "react";
import {ButtonConfig} from "../../../state/configuration/types/ButtonConfig";
import {DefaultButton, DirectionalHint, IconButton, useTheme} from "@fluentui/react";
import {useWatch} from "../../../watchables/react/useWatch";
import {StyledTooltipHost} from "../StyledToolTipHost";
import {usePersistentMemo} from "../../../utils/usePersistentMemo";
import {v4 as uuid} from "uuid";
import {
    PanelConfig,
    PanelConfigViewState,
} from "../../../state/configuration/types/PanelConfig";
import {ConfigTypeComp} from "./ConfigTypeComp";
import {useDragStart} from "../../../utils/useDragStart";
import {useViewManager} from "../../providers/ViewManagerContext";
import {StyledIconButton} from "../StyledIconButton";
import {Observer} from "../../../watchables/Observer";
import {Derived} from "../../../watchables/Derived";

export const PanelConfigComp: FC<{value: PanelConfig}> = ({value}) => {
    const watch = useWatch();
    const label = watch(value.label);
    const icon = watch(value.icon);
    const view = value.view;

    const views = useViewManager();
    const onClick = useCallback(() => {
        if (views.isVisible(view).get()) views.close(view).commit();
        else views.open(view).commit();
    }, []);

    useEffect(() => {
        const targetPanelExists = views.targetPanelExists(view);
        const observer = new Observer(
            new Derived(watch => {
                let mode = watch(value.autoOpen);
                if (mode == "always") {
                    return true;
                } else if (mode == "ifPanelExists") {
                    return watch(targetPanelExists);
                }
                return false;
            })
        );
        observer.add((shouldOpen, shouldOpenBefore) => {
            const init = shouldOpenBefore == undefined;
            if (shouldOpen != shouldOpenBefore && shouldOpen && !views.isOpen(view).get())
                views.open(view, h => h, init).commit();
        }, true);

        return () => observer.destroy();
    }, [value]);

    const id = usePersistentMemo(() => uuid(), []);
    let buttonEl = icon ? (
        <StyledIconButton
            icon={icon}
            tooltipId={id}
            name={icon}
            size={40}
            onClick={onClick}
        />
    ) : (
        <DefaultButton text={label} onClick={onClick} />
    );

    const ref = useDragStart((position, offset) => {
        const layout = views.layoutState;
        const container = layout.allTabPanels
            .get()
            .find(c => c.tabs.some(({id}) => id == view.ID));

        layout
            .setDraggingData({
                position,
                offset,
                targetId: view.ID,
                removeFromPanelId: container?.id,
                preview: buttonEl,
            })
            .commit();
    });

    return (
        <div ref={ref} style={{display: "flex", flexFlow: "inherit"}}>
            {icon && label ? (
                <StyledTooltipHost
                    directionalHint={DirectionalHint.leftCenter}
                    id={id}
                    content={label}>
                    {buttonEl}
                </StyledTooltipHost>
            ) : (
                buttonEl
            )}
        </div>
    );
};

export const PanelConfigView: FC<{state: PanelConfigViewState}> = ({state}) => {
    const watch = useWatch();
    const config = watch(state.config);
    const theme = useTheme();
    return (
        <div style={{margin: theme.spacing.s1}}>
            <ConfigTypeComp value={config} />
        </div>
    );
};
