import {CSSObject} from "@emotion/css";
import {ITheme, createTheme} from "@fluentui/react";
import Color from "color";

export const darkTheme = createTheme({
    palette: {
        themePrimary: "#45aeff",
        themeLighterAlt: "#03070a",
        themeLighter: "#0b1c29",
        themeLight: "#15344d",
        themeTertiary: "#296999",
        themeSecondary: "#3d99e0",
        themeDarkAlt: "#57b6ff",
        themeDark: "#72c2ff",
        themeDarker: "#97d2ff",
        neutralLighterAlt: "#31322b",
        neutralLighter: "#393a32",
        neutralLight: "#46483f",
        neutralQuaternaryAlt: "#4f5046",
        neutralQuaternary: "#55574d",
        neutralTertiaryAlt: "#727468",
        neutralTertiary: "#f5f5f5",
        neutralSecondary: "#f6f6f6",
        neutralPrimaryAlt: "#f8f8f8",
        neutralPrimary: "#f0f0f0",
        neutralDark: "#fbfbfb",
        black: "#fdfdfd",
        white: "#272822",
    },
    semanticColors: {
        errorText: "#ff6565",
    },
});

export const highlightTheme = (theme: ITheme): CSSObject => ({
    ".identifier": {
        color: "white",
    },
    ".glyph, .name, .location, .arrow, .prodLabel, .symLabelName": {
        color: "#bbbbbb",
    },
    ".string, .lit, .cilit": {
        color: "#dadb74",
    },
    ".number, .boolean, .escaped": {
        color: "#ae81ff",
    },
    ".collapse, .count, .label, .prodTag, .prodTag *": {
        color: "#747474",
    },
    ".context": {
        color: "#747474",
    },
    ".count, .label, .prodLabel, .symLabelName": {
        fontStyle: "italic",
    },
    ".greyOut, .greyOut *": {
        color: "#bbbbbb !important",
    },
    ".highlight": {
        color: theme.palette.themeDark,
    },
    ".keyword": {
        color: new Color(theme.palette.themeSecondary).lighten(0.4).toString(),
        whiteSpace: "nowrap",
    },
    ".annotation": {
        color: theme.palette.themeDarker,
    },
});
