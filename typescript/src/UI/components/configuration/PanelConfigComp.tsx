import React, {FC, useCallback, useMemo} from "react";
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
import {css} from "@emotion/css";
import {StyledIconButton} from "../StyledIconButton";
import {chain} from "../../../watchables/mutator/chain";

export const PanelConfigComp: FC<{value: PanelConfig}> = ({value}) => {
    const watch = useWatch();
    const theme = useTheme();
    const label = watch(value.label);
    const icon = watch(value.icon);
    const view = value.view;

    const views = useViewManager();
    const onClick = useCallback(() => {
        if (views.isVisible(view).get()) views.close(view).commit();
        else views.open(view).commit();
    }, []);

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
    return <ConfigTypeComp value={config} />;
};
