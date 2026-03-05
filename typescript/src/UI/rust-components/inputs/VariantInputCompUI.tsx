import React, {FC, useCallback, useMemo} from "react";
import {useWatch} from "../../../watchables/react/useWatch";
import {
    Checkbox,
    DirectionalHint,
    Dropdown,
    IconButton,
    IDropdownOption,
    IPivotItemProps,
    Pivot,
    PivotItem,
    Stack,
    useTheme,
} from "@fluentui/react";
import {NFC} from "../../../utils/_types/NFC";
import {IAriaRef} from "../_types/IAriaRef";
import {Component, ComponentVecWatchable, VariantInputComp} from "oxidd-vis-rust";
import {ICompUI} from "../_types/ICompUI";
import {IWatchable} from "../../../watchables/_types/IWatchable";
import {Derived} from "../../../watchables/Derived";
import {IWatcher} from "../../../watchables/_types/IWatcher";
import {useDerived} from "../../../watchables/react/useDerived";
import {StyledTooltipHost} from "../../components/StyledToolTipHost";
import {css} from "@emotion/css";
import {alignToText} from "../other/CompositeCompUI";
import {multiplySize} from "../../../utils/multiplySize";

export const VariantInputCompUI: NFC<{
    data: VariantInputComp;
    editData?: VariantInputComp;
    ChildComp: ICompUI;
    className?: string;
    aria?: IAriaRef;
}> = ({data, editData = data, ChildComp, className, aria}) => {
    const watch = useWatch();
    const options = watch(data.options);
    const selected = watch(data.selected);
    const disabled = watch(data.disabled);
    const theme = useTheme();

    const derivedDropdownOptions = useDerived(
        watch => getDropdownItems(options, watch),
        [options]
    );
    const derivedIconOptions = useDerived(
        watch => getIconSelectionItems(options, watch),
        [options]
    );
    const advancedOptions = useDerived(
        watch => getAdvancedSelectionItems(options, watch),
        [options]
    );

    const onDropdownChange = useCallback(
        (event: unknown, data: unknown, index: number) => {
            editData.select(index).commit();
        },
        [editData]
    );

    const dropdownOptions = watch(derivedDropdownOptions);
    if (dropdownOptions) {
        return (
            <Dropdown
                selectedKey={selected}
                onChange={onDropdownChange}
                aria-describedby={aria?.descriptionID}
                aria-labelledby={aria?.labelID}
                className={className}
                disabled={disabled}
                options={dropdownOptions}
            />
        );
    }

    const horizontal = watch(data.horizontal);
    const iconOptions = watch(derivedIconOptions);
    if (iconOptions) {
        return (
            <Pivot
                aria-describedby={aria?.descriptionID}
                aria-labelledby={aria?.labelID}
                className={`${className} ${css({"[role='tab']": {padding: 0, marginRight: 0}})}`}
                selectedKey={selected + ""}
                styles={{
                    root: {
                        ...(disabled ? {cursor: "not-allowed", opacity: 0.6} : undefined),
                        ...(horizontal == false
                            ? {display: "flex", flexDirection: "column"}
                            : undefined),
                    },
                }}
                onLinkClick={e =>
                    editData.select(parseInt(e!.props.itemKey!) as number).commit()
                }>
                {iconOptions.map((item, i) => (
                    <PivotItem
                        itemIcon={item.icon}
                        itemKey={i + ""}
                        title={item.title}
                        onRenderItemLink={TooltipPivot}
                    />
                ))}
            </Pivot>
        );
    }

    const childGap = watch(data.gap);
    const align = watch(data.main_align);
    const perpendicularAlign = watch(data.perpendicular_align);
    const horizontalAlign = horizontal == true ? align : perpendicularAlign;
    const verticalAlign = horizontal == true ? perpendicularAlign : align;
    return (
        <Stack
            aria-describedby={aria?.descriptionID}
            aria-labelledby={aria?.labelID}
            className={className}
            tokens={{childrenGap: multiplySize(childGap, theme.spacing.s1)}}
            horizontal={horizontal}
            verticalAlign={alignToText(verticalAlign)}
            horizontalAlign={alignToText(horizontalAlign)}
            style={disabled ? {cursor: "not-allowed", opacity: 0.6} : undefined}>
            {watch(advancedOptions).map((item, i) => (
                <InputOption
                    name={item.name}
                    selected={i == selected}
                    onSelect={() => {
                        editData.select(i).commit();
                    }}>
                    <ChildComp data={item.child} />
                </InputOption>
            ))}
        </Stack>
    );
};

/**
 * Checks whether all options are text options, and if so returns the items
 */
function getDropdownItems(
    components: Component[],
    watch: IWatcher
): IDropdownOption<void>[] | undefined {
    const out = [];
    let i = 0;
    for (let comp of components) {
        const textComp = comp.as_text();
        if (textComp == undefined) return undefined;
        const text = watch(textComp.text);
        out.push({key: i, text});
        i++;
    }
    return out;
}

/**
 * Checks whether all items are icon buttons, and f so returns the items
 */
function getIconSelectionItems(
    components: Component[],
    watch: IWatcher
): {icon: string; title?: string}[] | undefined {
    const out = [];
    let i = 0;
    for (let comp of components) {
        const textComp = comp.as_button();
        if (textComp == undefined) return undefined;
        const icon = watch(textComp.icon);
        if (icon == undefined) return undefined;
        const title = watch(textComp.text);
        out.push({icon, title});
        i++;
    }
    return out;
}

/**
 * Retrieves the advanced selection items, each with possibly a name
 */
function getAdvancedSelectionItems(
    components: Component[],
    watch: IWatcher
): {name?: JSX.Element; child: Component}[] {
    const out = [];
    let i = 0;
    for (let comp of components) {
        const labelComp = comp.as_label();
        if (labelComp) {
            const name = watch(labelComp.label);
            out.push({name: <>{name}</>, child: watch(labelComp.input)});
        } else {
            out.push({child: comp});
        }
    }
    return out;
}

/** An input option component */
export const InputOption: FC<{
    selected: boolean;
    onSelect: () => void;
    name?: JSX.Element;
}> = ({children, selected, onSelect, name}) => {
    const theme = useTheme();
    return (
        <div
            style={{
                overflow: "hidden",
                backgroundColor: theme.palette.neutralLighterAlt,
            }}>
            <div
                onClick={onSelect}
                style={{
                    backgroundColor: theme.palette.neutralLighter,
                    padding: 10,
                    display: "flex",
                    alignItems: "center",
                    gap: 5,
                    fontSize: 16,
                    fontWeight: 600,
                    cursor: "pointer",
                }}>
                <Checkbox checked={selected} />
                {name}
            </div>
            <div
                style={{
                    padding: 10,
                }}>
                {children}
            </div>
        </div>
    );
};

const TooltipPivot = (
    link?: IPivotItemProps,
    defaultRenderer?: (link?: IPivotItemProps) => JSX.Element | null
): JSX.Element | null => {
    console.log(link);
    if (!link || !defaultRenderer) {
        return null;
    }

    return (
        <StyledTooltipHost
            directionalHint={DirectionalHint.leftCenter}
            content={link.title}>
            <span style={{marginLeft: 12, marginRight: 12}}>{defaultRenderer(link)}</span>
        </StyledTooltipHost>
    );
};
