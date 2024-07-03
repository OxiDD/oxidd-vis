import React, {FC, useEffect, useMemo, useRef, CSSProperties} from "react";
import {IDragDataInput, ITabsHeaderProps} from "../_types/props/ITabsHeaderProps";
import {
    CommandButton,
    IButtonProps,
    IconButton,
    Pivot,
    PivotItem,
    mergeStyles,
    useTheme,
} from "@fluentui/react";
import {css} from "@emotion/css";
import {useDragStart} from "../../utils/useDragStart";

export const TabsHeader: FC<
    ITabsHeaderProps & {ExtraHeader?: FC<{onClose: () => void}>}
> = ({
    onClose,
    tabs,
    dragging,
    onCloseTab,
    onDrop,
    onSelectTab,
    selectedTab,
    onDragStart,
    ExtraHeader,
}) => {
    const OverflowAndRemainder = useMemo(
        () =>
            getOverflowAndRemainder(
                dragging,
                onDrop,
                tabs.length == 0,
                onClose,
                ExtraHeader
            ),
        [dragging, tabs.length == 0, ExtraHeader]
    );
    const theme = useTheme();

    return (
        <div
            className={`layout-tabs-header ${pivotStyle}`}
            style={{display: "flex", backgroundColor: theme.palette.neutralLighter}}>
            <Pivot
                style={{flexGrow: 1, flexShrink: 1, minWidth: 0}}
                focusZoneProps={{style: {display: "flex", minHeight: 44}}}
                overflowBehavior="menu"
                overflowAriaLabel="more items"
                selectedKey={selectedTab}
                headersOnly={true}
                overflowButtonAs={OverflowAndRemainder}
                onLinkClick={item =>
                    item?.props.itemKey && onSelectTab(item.props.itemKey)
                }>
                {tabs.map(({id, name, forceOpen, onTabContext}, i) => (
                    <PivotItem
                        key={id}
                        headerText={name}
                        headerButtonProps={{
                            onContextMenu: onTabContext as any,
                        }}
                        itemKey={id}
                        onRenderItemLink={() => (
                            <Tab
                                first={i == 0}
                                name={name}
                                dragging={dragging}
                                forceOpen={forceOpen}
                                onDrop={side =>
                                    onDrop(tabs[i + (side == "after" ? 1 : 0)]?.id)
                                }
                                onClose={() => onCloseTab(id)}
                                selected={id == selectedTab}
                                onDrag={data =>
                                    onDragStart({
                                        ...data,
                                        targetId: id,
                                    })
                                }
                            />
                        )}
                    />
                ))}
            </Pivot>
        </div>
    );
};
const pivotStyle = mergeStyles({
    "& span": {
        position: "initial !important",
    },
});

const getOverflowAndRemainder: (
    dragging: boolean,
    onDrop: () => void,
    empty: boolean,
    onClose: () => void,
    ExtraHeader?: FC<{onClose: () => void}>
) => FC<IButtonProps> = (dragging, onDrop, empty, onClose, ExtraHeader) => props =>
    (
        <>
            <div style={{flexGrow: 1, position: "relative"}}>
                {dragging && (
                    <DropContainer
                        style={{
                            left: empty ? 0 : -dropOffset - dropWidth,
                            borderLeftWidth: dropWidth,
                        }}
                        onDrop={onDrop}
                    />
                )}
            </div>
            <CommandButton {...props} menuIconProps={{iconName: "ChevronDown"}} />
            {ExtraHeader && <ExtraHeader onClose={onClose} />}
        </>
    );

export const Tab: FC<{
    first: boolean;
    onClose: () => void;
    name: string;
    selected: boolean;
    onDrag: (input: Omit<IDragDataInput, "targetId">) => void;
    dragging: boolean;
    forceOpen: boolean;
    onDrop: (side: "before" | "after") => void;
}> = props => {
    const {onClose, name, selected, onDrag, dragging, onDrop, first, forceOpen} = props;
    const theme = useTheme();
    const ref = useDragStart((position, offset) => {
        onDrag({
            position,
            offset,
            preview: (
                <Pivot headersOnly style={{backgroundColor: theme.palette.white}}>
                    <PivotItem onRenderItemLink={() => <Tab {...props} />} />
                </Pivot>
            ),
        });
    });

    return (
        <>
            {name}
            {selected && !forceOpen && (
                <>
                    <span style={{display: "inline-block", width: 20}} />
                    <div style={{position: "absolute", zIndex: 100, top: 5, right: 0}}>
                        <IconButton
                            styles={{
                                icon: {width: "auto"},
                                root: {width: "auto", padding: 0},
                            }}
                            iconProps={{iconName: "Cancel"}}
                            onClick={onClose}
                        />
                    </div>
                </>
            )}
            <div
                style={{position: "absolute", left: 0, top: 0, right: 0, bottom: 0}}
                ref={ref}>
                {dragging && (
                    <>
                        <DropContainer
                            style={{
                                left: first ? 0 : -dropOffset - dropWidth,
                                right: "50%",
                                borderLeftWidth: dropWidth,
                            }}
                            onDrop={() => onDrop("before")}
                        />
                        <DropContainer
                            style={{
                                left: "50%",
                                right: -dropOffset - dropWidth,
                                borderRightWidth: dropWidth,
                            }}
                            onDrop={() => onDrop("after")}
                        />
                    </>
                )}
            </div>
        </>
    );
};

const dropWidth = 4;
const dropOffset = 2;

// Drop target styling
const DropContainer: FC<{style: CSSProperties; onDrop: () => void}> = ({
    style,
    onDrop,
}) => {
    const theme = useTheme();
    const dropColor = theme.palette.themePrimary;
    return (
        <div
            className={css({
                borderWidth: 0,
                borderStyle: "none",
                "&:hover": {
                    borderStyle: "solid",
                },

                zIndex: 100,
                position: "absolute",
                left: 0,
                top: 0,
                right: 0,
                bottom: 0,
                borderColor: dropColor,
                ...style,
            })}
            onMouseUp={onDrop}
        />
    );
};
