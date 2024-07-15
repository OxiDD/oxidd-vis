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
