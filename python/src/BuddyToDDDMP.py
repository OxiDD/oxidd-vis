
def BuddyToDDDMP(data: str, vars: str):
    lines = data.split("\n")
    perm_ids = lines[1].strip().split(" ")
    ids = [int(v) for v in perm_ids]
    ids.sort()
    nodes = lines[2::]
    root_id = nodes[-1].split(" ")[0]
    if vars:
        var_names = " ".join(vars.split("\n"))
    else:
        var_names = perm_ids

    nl = "\n"   
    out = f""".ver DDMP-2.0
.mode A
.varinfo 4
.nnodes {len(nodes)}
.nvars {len(ids)}
.nsuppvars {len(ids)}
.suppvarnames {var_names}
.orderedvarnames {var_names}
.ids {" ".join([str(v) for v in ids])}
.permids {" ".join(perm_ids)}
.nroots 1
.rootids {root_id}
.rootnames {root_id}
.nodes
0 F 0 0
1 T 0 0
{nl.join(nodes)}
.end"""
    return out
    