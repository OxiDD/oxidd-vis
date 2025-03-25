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

// This light theme is made later, colors are picked arbitrarily to look decent without changing the rest of the UI
export const lightTheme = createTheme({
    palette: {
        themePrimary: "#45aeff",
        themeLighterAlt: "#03070a",
        themeLighter: "#0b1c29",
        themeLight: "#15344d",
        themeTertiary: "#c7e7ff",
        themeSecondary: "#106ebe",
        themeDarkAlt: "#57b6ff",
        themeDark: "#57b6ff",
        themeDarker: "#106ebe",
        neutralLighterAlt: "#f4f4f4",
        neutralLighter: "#e9e9e9",
        neutralLight: "#ffffff",
        neutralQuaternaryAlt: "#d6D6d6",
        neutralQuaternary: "#dddddd",
        neutralTertiaryAlt: "#b7b7b7",
        neutralTertiary: "#55574d",
        neutralSecondary: "#4f5046",
        neutralPrimaryAlt: "#46483f",
        neutralPrimary: "#393a32",
        neutralDark: "#31322b",
        black: "#272822",
        white: "#fdfdfd",
    },
    semanticColors: {
        errorText: "#bb2222",
    },
});
