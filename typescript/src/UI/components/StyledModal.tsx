import React, {FC, useMemo} from "react";
import {
    FontWeights,
    IModalProps,
    ISpinButtonProps,
    IconButton,
    Modal,
    SpinButton,
    mergeStyleSets,
    useTheme,
} from "@fluentui/react";
import {useId} from "@fluentui/react-hooks";

export const StyledModal: FC<IModalProps & {title: string}> = props => {
    const titleId = useId("title");
    const theme = useTheme();
    const contentStyles = useMemo(
        () =>
            mergeStyleSets({
                container: {
                    display: "flex",
                    flexFlow: "column nowrap",
                    alignItems: "stretch",
                },
                header: [
                    // eslint-disable-next-line deprecation/deprecation
                    theme.fonts.xLargePlus,
                    {
                        flex: "1 1 auto",
                        borderTop: `4px solid ${theme.palette.themePrimary}`,
                        color: theme.palette.neutralPrimary,
                        display: "flex",
                        alignItems: "center",
                        fontWeight: FontWeights.semibold,
                        padding: "12px 12px 14px 24px",
                    },
                ],
                heading: {
                    color: theme.palette.neutralPrimary,
                    fontWeight: FontWeights.semibold,
                    fontSize: "inherit",
                    margin: "0",
                },
                body: {
                    flex: "4 4 auto",
                    padding: "0 24px 24px 24px",
                    overflowY: "hidden",
                    selectors: {
                        p: {margin: "14px 0"},
                        "p:first-child": {marginTop: 0},
                        "p:last-child": {marginBottom: 0},
                    },
                },
            }),
        []
    );
    const iconButtonStyles = useMemo(
        () => ({
            root: {
                color: theme.palette.neutralPrimary,
                marginLeft: "auto",
                marginTop: "4px",
                marginRight: "2px",
            },
            rootHovered: {
                color: theme.palette.neutralDark,
            },
        }),
        []
    );

    return (
        <Modal {...props} className={`${contentStyles} ${props.className}`}>
            <div className={contentStyles.header}>
                <h2 className={contentStyles.heading} id={titleId}>
                    {props.title}
                </h2>
                <IconButton
                    styles={iconButtonStyles}
                    iconProps={{iconName: "Cancel"}}
                    ariaLabel="Close popup modal"
                    onClick={() => props.onDismiss?.()}
                />
            </div>
            <div className={contentStyles.body}>{props.children}</div>
        </Modal>
    );
};
