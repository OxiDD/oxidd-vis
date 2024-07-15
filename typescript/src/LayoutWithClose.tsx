import React, {FC, createContext, useContext} from "react";
import {ILayoutProps} from "./layout/_types/ILayoutProps";
import {ILayoutComponents} from "./layout/_types/ILayourComponents";
import {DefaultLayout} from "./layout/DefaultLayout";
import {IWatchable} from "./watchables/_types/IWatchable";
import {Constant} from "./watchables/Constant";
import {ITabsHeaderProps} from "./layout/_types/props/ITabsHeaderProps";
import {useWatch} from "./watchables/react/useWatch";
import {TabsHeader} from "./layout/styledComponents/Tabsheader";
import {ActionButton, CommandButton, IconButton, useTheme} from "@fluentui/react";
import {css} from "@emotion/css";

/**
 * The layout entry component, where styling components are already provided
 */
export const LayoutWithClose: FC<
    Omit<ILayoutProps, "components"> & {
        components?: Partial<ILayoutComponents>;
        panelClosable: IWatchable<IPanelClosable>;
    }
> = ({panelClosable, components, ...props}) => {
    return (
        <panelClosableContext.Provider value={panelClosable}>
            <DefaultLayout
                {...props}
                components={{TabsHeader: ClosableTabsHeader, ...components}}
            />
        </panelClosableContext.Provider>
    );
};

/** A type specifying whether panels are closable */
export type IPanelClosable = "always" | "never" | "whenEmpty";

/** The context that specifies whether panels should be closable */
export const panelClosableContext = createContext<IWatchable<IPanelClosable>>(
    new Constant("always")
);

/** The tabs header with an extra close button for the entire panel depending on the panelClosableContext */
export const ClosableTabsHeader: FC<ITabsHeaderProps> = props => {
    const watch = useWatch();
    const panelClosable = watch(useContext(panelClosableContext));

    const showClose =
        panelClosable == "always" ||
        (panelClosable == "whenEmpty" && props.tabs.length == 0);

    return (
        <TabsHeader {...props} ExtraHeader={showClose ? CloseHeaderButton : undefined} />
    );
};

/** The panel close button */
export const CloseHeaderButton: FC<{onClose: () => void}> = ({onClose}) => {
    const theme = useTheme();
    return (
        <IconButton
            title="close panel"
            onClick={onClose}
            className={css({
                alignSelf: "stretch",
                height: "auto",
                width: 44,
                display: "flex",
                justifyContent: "center",
                ":hover": {
                    backgroundColor: theme.palette.neutralLighterAlt,
                },
            })}
            iconProps={{iconName: "Cancel"}}
        />
    );
};
