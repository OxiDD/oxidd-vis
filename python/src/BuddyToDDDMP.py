
def BuddyToDDDMP(data: str, vars: str):
    lines = data.strip().split("\n")
    indices = [int(i) for i in lines[1].strip().split(" ")]
    perm_ids = [0] * len(indices)
    for layer in range(0, len(indices)):
        perm_ids[indices[layer]] = layer
    ids = [int(v) for v in perm_ids]
    ids.sort()
    if vars:
        var_names = vars.strip().split("\n")
    else:
        var_names = [f'x{x}' for x in ids]
    ordered_var_names = [var_names[int(i)] for i in perm_ids]

    root_id = lines[-1].split(" ")[0]
    node_data = [line.strip().split(" ") for line in lines[2::]]
    nodes = [f'{n[0]} {str(indices[int(n[1])])} {n[3]} {n[2]}' for n in node_data]

    nl = "\n"   
    out = f""".ver DDMP-2.0
.mode A
.varinfo 4
.nnodes {len(nodes)}
.nvars {len(ids)}
.nsuppvars {len(ids)}
.suppvarnames {" ".join(ordered_var_names)}
.orderedvarnames {" ".join(var_names)}
.ids {" ".join([str(v) for v in ids])}
.permids {" ".join([str(v) for v in perm_ids])}
.nroots 1
.rootids {root_id}
.rootnames f
.nodes
0 F 0 0
1 T 0 0
{nl.join(nodes)}
.end"""
    return out
    