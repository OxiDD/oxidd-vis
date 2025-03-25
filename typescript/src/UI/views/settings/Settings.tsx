import React, {FC} from "react";
import {SettingsState} from "../../../state/SettingsState";
import {useWatch} from "../../../watchables/react/useWatch";
import {Toggle} from "@fluentui/react";

export const Settings: FC<{settings: SettingsState}> = ({settings}) => {
    const watch = useWatch();
    return (
        <div>
            Delete unused panels:
            <Toggle
                checked={watch(settings.layout.deleteUnusedPanels)}
                onChange={(_, checked) =>
                    settings.layout.deleteUnusedPanels.set(checked ?? false).commit()
                }
            />
        </div>
    );
};
