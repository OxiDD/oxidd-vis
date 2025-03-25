import {Icon, Link, useTheme} from "@fluentui/react";
import React, {FC} from "react";
import {ViewState} from "../../../state/views/ViewState";
import {AppState} from "../../../state/AppState";
import {css} from "@emotion/css";
import {ViewContainer} from "../../components/layout/ViewContainer";
import {CenteredContainer} from "../../components/layout/CenteredContainer";

export const Info: FC<{app: AppState}> = ({app}) => {
    const theme = useTheme();
    const link = (text: string, openView: ViewState) => (
        <Link onClick={() => app.views.open(openView).commit()}>{text}</Link>
    );

    return (
        <CenteredContainer>
            <h1 style={{color: theme.palette.themePrimary, fontSize: 40, marginTop: 10}}>
                <Icon
                    iconName="GitGraph"
                    styles={{
                        root: {marginRight: 10, fontSize: 45, verticalAlign: "bottom"},
                    }}
                />
                BDD-viz
            </h1>
            <p>
                BDD-viz (temporary name) is a Binary Decision Diagram (BDD) visualization
                tool. It is still a work in progress, meaning that there are many features
                still missing. Some of these features already have traces in the design,
                but are not fully there yet. Are you experiencing any difficulties or have
                suggestions for improvement? Please send me an email at{" "}
                <Link href="mailto:t.m.k.v.krieken@tue.nl">t.m.k.v.krieken@tue.nl</Link> .
            </p>

            <h2>Loading diagrams</h2>
            <p>
                There are two ways to add diagrams:{" "}
                <ul>
                    <li>Local: Manually adding a diagram + loading nodes from a file</li>
                    <li>Remote: Adding a remote diagram source</li>
                </ul>
            </p>
            <h3>Local</h3>
            <p>
                To add a local diagram, simply open the{" "}
                {link("diagrams panel", app.diagrams)} and click "Add local DD". This
                should adds a shared diagram, which you can now add content into using
                either "Load from dddump" or 'Load from Buddy'. Here you can select either
                the file or text contents, and load the contents. After the contents
                loaded, a new section should appear in the diagram, and clicking it opens
                the visualization.
            </p>
            <h3>Remote</h3>
            <p>
                To add a remote diagram source, simply open the{" "}
                {link("diagrams panel", app.diagrams)} and click "Add diagram source".
                This opens up a window in which you can select the host of the diagrams
                you want to show. In most cases, this host will be the default entered
                address (at which you are also accessing the tool). After the host is
                added, it will be regularly checked for diagram updates and reflect them.{" "}
                <br />
                Here you can also select "Automatically open diagrams" to open a tab in
                which the diagrams will be opened when created.
                <br />
            </p>
            <p>
                There are several ways to add a diagram to the default remote host. If you
                make use of the course provided Buddy python wrapper, there is a python
                function <code>manager.visualize(root, "diagram_name")</code> that can be
                invoked. This will then show the diagram in this tool. Similarly the Oxidd
                python wrapper obtainable from{" "}
                <Link href="https://github.com/TarVK/oxidd">github.com/TarVK/oxidd</Link>{" "}
                contains a function <code>manager.visualize(root, "diagram_name")</code>{" "}
                that behaves the same. Finally, this Oxidd fork also contains{" "}
                <Link href="https://github.com/TarVK/oxidd/blob/main/crates/oxidd-dump/src/visualize.rs">
                    a visualize function
                </Link>{" "}
                that can be used to visualize the diagram when writing code in Rust.
            </p>

            <h2>Features</h2>
            <p>
                The tool is not finished, but already includes several useful features.
                These will be briefly introduced here.
            </p>
            <h3>Tabs</h3>
            <p>
                The layout of the tool can be customized by dragging tabs around. Simply
                drag a tab and drop it in one of the highlighted blue areas to split the
                corresponding section into two. This allows for viewing several diagrams
                side by side.
            </p>
            <h3>Terminal hiding</h3>
            <p>
                By default the false terminals are hidden in the diagram. This means that
                if a node only has 1 outgoing edge, the other edge goes to the false node.
                For either terminal, a dropdown in the bottom right of the diagram
                controls whether it is visible. Instead of showing it as a single node,
                you can also choose to duplicate it once per parent node.
            </p>
            <h3>Node grouping</h3>
            <p>
                To deal with large diagrams, nodes can be grouped together. By default, if
                a loaded diagram is too large to effectively display, its nodes remain
                grouped in a single group. The tools in the top right can be used to
                either group a selection of nodes (drag on screen), or ungroup the
                children of a selection of nodes.
            </p>

            {/* <p>Settings: {link("settings", app.settings)}</p> */}
        </CenteredContainer>
    );
};
