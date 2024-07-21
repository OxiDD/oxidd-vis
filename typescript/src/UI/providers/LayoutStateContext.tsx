import {createContext, useContext} from "react";
import {LayoutState} from "../../layout/LayoutState";

/** The context for layout data, mainly used for setting dragging data */
export const LayoutStateContext = createContext(new LayoutState());

/** The provider for layout data */
export const LayoutStateProvider = LayoutStateContext.Provider;

/** The hook to get the layout state */
export const useLayoutState = () => useContext(LayoutStateContext);
