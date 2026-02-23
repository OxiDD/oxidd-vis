import React, {FC, createContext, forwardRef, useContext, useEffect, useRef} from "react";
import {ILayoutProps} from "../../../layout/_types/ILayoutProps";
import {ILayoutComponents} from "../../../layout/_types/ILayourComponents";
import {DefaultLayout} from "../../../layout/DefaultLayout";
import {IWatchable} from "../../../watchables/_types/IWatchable";
import {Constant} from "../../../watchables/Constant";
import {ITabsHeaderProps} from "../../../layout/_types/props/ITabsHeaderProps";
import {useWatch} from "../../../watchables/react/useWatch";
import {TabsHeader} from "../../../layout/styledComponents/Tabsheader";
import {IconButton, useTheme} from "@fluentui/react";
import {css} from "@emotion/css";
import {useViewManager} from "../../providers/ViewManagerContext";
import {ITabsContentProps} from "../../../layout/_types/props/ITabsContentProps";
import {TabTarget, TabsContent} from "../../../layout/styledComponents/TabsContent";
import {mergeRefs} from "react-merge-refs";

/**
 * The layout entry component, where styling components are already provided
 */
export const CustomLayout: FC<
    Omit<ILayoutProps, "components"> & {
        components?: Partial<ILayoutComponents>;
        panelClosable: IWatchable<IPanelClosable>;
    }
> = ({panelClosable, components, ...props}) => {
    return (
        <panelClosableContext.Provider value={panelClosable}>
            <DefaultLayout
                {...props}
                components={{
                    TabsHeader: ClosableTabsHeader,
                    TabsContent: TabContentWithSelect,
                    ...components,
                }}
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

/** The tabs header with an extra close button for the entire panel depending on the panelClosableContext, which also selects associated tabs when clicked */
export const ClosableTabsHeader: FC<ITabsHeaderProps> = props => {
    const watch = useWatch();
    const panelClosable = watch(useContext(panelClosableContext));
    const viewManager = useViewManager();

    const showClose =
        props.onClose &&
        (panelClosable == "always" ||
            (panelClosable == "whenEmpty" && props.tabs.length == 0));

    return (
        <TabsHeader
            {...props}
            onSelectTab={id => viewManager.focus(id).commit()}
            ExtraHeader={showClose ? CloseHeaderButton : undefined}
        />
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
                marginLeft: -10,
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

/** The tabs content element */
export const TabContentWithSelect: FC<ITabsContentProps> = ({contents}) => {
    const viewManager = useViewManager();
    return (
        <>
            {contents.map(({id, element, selected}) => (
                <TabTargetWithClick
                    key={id}
                    selected={selected}
                    element={element}
                    onClick={() => viewManager.focus(id).commit()}
                />
            ))}
        </>
    );
};

/** Use a dom event listener to get around the fact that tab contents are rendered using portals, and hence the react virtual DOM bubbles differently than the html DOM */
export const TabTargetWithClick = forwardRef<
    HTMLDivElement,
    {selected: boolean; element: HTMLElement; onClick: () => void}
>(({onClick, ...rest}, fRef) => {
    const lRef = useRef<HTMLDivElement>(null);
    useEffect(() => {
        const el = lRef.current;
        if (!el) return;
        if (!onClick) return;

        el.addEventListener("mousedown", onClick);
        return () => el.removeEventListener("mousedown", onClick);
    }, [lRef]);
    return <TabTarget {...rest} ref={mergeRefs([lRef, fRef])} />;
});
