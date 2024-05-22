package policy

default allow = false

path := data["resource-path"]
input_tcb := input["tcb-status"]
parts_resource_path := split(path, "/")

allow {
        allowed := {path, "*"}
        some key
        contains(key, ".authorized_res.")
        input_tcb[key] == allowed[_]
}

allow {
        some key
        contains(key, ".authorized_res.")
        count(parts_resource_path) == 3
        parts_authorized_res := split(input_tcb[key], "/")
        count(parts_authorized_res) == 3
        part_matches_or_wildcard(parts_resource_path[0], parts_authorized_res[0])
        part_matches_or_wildcard(parts_resource_path[1], parts_authorized_res[1])
        part_matches_or_wildcard(parts_resource_path[2], parts_authorized_res[2])
}

part_matches_or_wildcard(resource_part, authorized_part) {
        allowed := {resource_part, "*"}
        authorized_part == allowed[_]
}