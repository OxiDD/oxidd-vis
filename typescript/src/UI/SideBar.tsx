import {DirectionalHint, IconButton, TooltipHost, useTheme} from "@fluentui/react";
import React, {FC, useCallback} from "react";
import {useId} from "@fluentui/react-hooks";
import {useDragStart} from "../utils/useDragStart";
import {css} from "@emotion/css";
import {StyledTooltipHost} from "./components/StyledToolTipHost";
import {GithubIcon} from "./components/GithubIcon";
import {AppState} from "../state/AppState";
import {useWatch} from "../watchables/react/useWatch";
import {ViewState} from "../state/views/ViewState";

const size = 50;
export const Sidebar: FC<{state: AppState; projectUrl: string}> = ({
    state,
    projectUrl,
}) => {
    const watch = useWatch();
    const theme = useTheme();
    const tabs = state.tabs;
    const githubId = useId("github");

    const themeId = useId("theme");
    const darkMode = watch(state.settings.global).darkMode;

    return (
        <>
            <div
                style={{
                    width: size,
                    backgroundColor: theme.palette.neutralLight,
                    display: "flex",
                    flexDirection: "column",
                }}>
                {tabs.map(props => (
                    <SidebarButton key={props.name} appState={state} {...props} />
                ))}

                <div style={{flexGrow: 1}} />
                <StyledTooltipHost
                    content={`Enable ${darkMode ? "light" : "dark"} mode`}
                    directionalHint={DirectionalHint.rightCenter}
                    id={themeId}>
                    <IconEl
                        tooltipId={themeId}
                        name="dark mode"
                        icon={darkMode ? "ClearNight" : "Sunny"}
                        onClick={() =>
                            state.settings.global
                                .set({
                                    ...state.settings.global.get(),
                                    darkMode: !darkMode,
                                })
                                .commit()
                        }
                    />
                </StyledTooltipHost>
                <StyledTooltipHost
                    content="Github repository"
                    directionalHint={DirectionalHint.rightCenter}
                    id={githubId}>
                    <IconButton
                        aria-describedby={githubId}
                        className={css({
                            width: size,
                            height: size,
                            backgroundColor: theme.palette.neutralLight,
                        })}
                        onRenderIcon={() => (
                            <GithubIcon
                                width={size * 0.55}
                                color={theme.palette.neutralPrimary}
                                hoverColor={theme.palette.themePrimary}
                            />
                        )}
                        aria-label="Github"
                        href={projectUrl}
                    />
                </StyledTooltipHost>
            </div>
            <div
                style={{
                    minWidth: 10,
                    boxShadow: "inset #0000004d 0px 0px 6px 2px",
                }}
            />
        </>
    );
};

export const SidebarButton: FC<{
    name: string;
    icon: string;
    view: ViewState;
    openIn?: string;
    appState: AppState;
}> = ({name, icon, view, appState, openIn}) => {
    const theme = useTheme();
    const tooltipId = useId(name);
    const watch = useWatch();
    const views = appState.views;
    const onClick = useCallback(() => {
        if (views.isVisible(view).get()) views.close(view).commit();
        else
            views
                .open(view, [
                    {targetId: openIn ?? "sidebar", targetType: "panel"},
                    {createId: openIn ?? "sidebar", weightRatio: 0.7, side: "west"},
                ])
                .commit();
    }, []);
    const isVisible = watch(views.isVisible(view));

    const iconEl = (
        <IconEl
            tooltipId={tooltipId}
            isVisible={isVisible}
            name={name}
            icon={icon}
            onClick={onClick}
        />
    );
    const ref = useDragStart((position, offset) => {
        const layout = appState.views.layoutState;
        const container = layout.allTabPanels
            .get()
            .find(c => c.tabs.some(({id}) => id == view.ID));
        layout
            .setDraggingData({
                position,
                offset,
                targetId: view.ID,
                removeFromPanelId: container?.id,
                preview: iconEl,
            })
            .commit();
    });

    return (
        <StyledTooltipHost
            content={name}
            directionalHint={DirectionalHint.rightCenter}
            id={tooltipId}>
            <div ref={ref}>{iconEl}</div>
        </StyledTooltipHost>
    );
};

const IconEl: FC<{
    tooltipId: string;
    isVisible?: boolean;
    name: string;
    icon: string;
    onClick: () => void;
}> = ({tooltipId, isVisible = false, name, icon, onClick}) => {
    const theme = useTheme();
    return (
        <IconButton
            aria-describedby={tooltipId}
            className={css({
                width: size,
                height: size,
                color: theme.palette.neutralPrimary,
                backgroundColor: isVisible
                    ? theme.palette.neutralLighterAlt
                    : theme.palette.neutralLight,
            })}
            iconProps={{iconName: icon, style: {fontSize: size * 0.5}}}
            aria-label={name}
            onClick={onClick}
        />
    );
};
