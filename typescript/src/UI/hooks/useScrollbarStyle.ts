import {CSSObject} from "@emotion/css";
import {useTheme} from "@fluentui/react";

/**
 * Retrieves scrollbar styling
 * @returns The styling
 */
export function useScrollbarStyle(): CSSObject {
    const theme = useTheme();
    return {
        /* width */
        "&::-webkit-scrollbar": {
            width: "10px",
            height: "10px",
        },

        /* Track */
        "&::-webkit-scrollbar-track": {
            background: theme.palette.white,
            borderLeft: `1px solid ${theme.palette.neutralLighter}`,
        },

        /* Handle */
        "&::-webkit-scrollbar-thumb": {
            background: theme.palette.neutralQuaternaryAlt,
        },

        /* Handle on hover */
        "&::-webkit-scrollbar-thumb:hover": {
            background: theme.palette.neutralTertiaryAlt,
        },
    };
}
