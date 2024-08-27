import {createContext, useContext} from "react";
import {ToolbarState} from "../../state/toolbar/ToolbarState";

/** The context for the toolbar */
export const ToolbarContext = createContext(new ToolbarState());

/** The provider for the toolbar */
export const ToolbarProvider = ToolbarContext.Provider;

/** The hook to get the toolbar */
export const useToolbar = () => useContext(ToolbarContext);
