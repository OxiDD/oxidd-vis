import {createContext, useContext} from "react";
import {AppState} from "../../state/AppState";

/** The context for all app data */
export const AppStateContext = createContext(null as any as AppState);

/** The provider for app data */
export const AppStateProvider = AppStateContext.Provider;

/** The hook to get the app state */
export const useAppState = () => useContext(AppStateContext);
