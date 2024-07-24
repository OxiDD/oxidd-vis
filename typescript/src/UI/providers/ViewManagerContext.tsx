import {createContext, useContext} from "react";
import {ViewManager} from "../../state/views/ViewManager";

/** The context for the view manager */
export const ViewManagerContext = createContext(null as any as ViewManager);

/** The provider for the view manager */
export const ViewManagerProvider = ViewManagerContext.Provider;

/** The hook to get the view manager */
export const useViewManager = () => useContext(ViewManagerContext);
