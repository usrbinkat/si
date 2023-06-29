load(
    "@prelude-si//:pnpm.bzl",
    _eslint = "eslint",
    _node_pkg_bin = "node_pkg_bin",
    _npm_bin = "npm_bin",
    _package_node_modules = "package_node_modules",
    _pnpm_lock = "pnpm_lock",
    _typescript_check = "typescript_check",
    _typescript_dist = "typescript_dist",
    _vite_app = "vite_app",
    _workspace_node_modules = "workspace_node_modules",
)

def eslint(
        eslint_bin = "eslint",
        directories = ["src"],
        package_node_modules = ":node_modules",
        visibility = ["PUBLIC"],
        **kwargs):
    if not rule_exists(eslint_bin):
        _npm_bin(
            name = eslint_bin,
            node_modules = package_node_modules,
            visibility = visibility,
        )

    _eslint(
        eslint = ":{}".format(eslint_bin),
        directories = directories,
        package_node_modules = package_node_modules,
        visibility = visibility,
        **kwargs,
    )

def node_pkg_bin(
        pkg_bin = "pkg",
        dist = ":dist",
        package_node_modules = ":node_modules",
        visibility = ["PUBLIC"],
        **kwargs):
    if not rule_exists(pkg_bin):
        _npm_bin(
            name = pkg_bin,
            node_modules = package_node_modules,
            visibility = visibility,
        )

    _node_pkg_bin(
        pkg = ":{}".format(pkg_bin),
        dist = dist,
        package_node_modules = package_node_modules,
        visibility = visibility,
        **kwargs,
    )

def npm_bin(
        visibility = ["PUBLIC"],
        node_modules = ":node_modules",
        **kwargs):
    _npm_bin(
        node_modules = node_modules,
        visibility = visibility,
        **kwargs,
    )

def package_node_modules(
        visibility = ["PUBLIC"],
        **kwargs):
    _package_node_modules(visibility = visibility, **kwargs)

def pnpm_lock(
        visibility = ["PUBLIC"],
        **kwargs):
    _pnpm_lock(visibility = visibility, **kwargs)

def typescript_check(
        tsc_bin = "tsc",
        tsc = ":tsc",
        package_node_modules = ":node_modules",
        visibility = ["PUBLIC"],
        **kwargs):
    if not rule_exists(tsc_bin):
        _npm_bin(
            name = tsc_bin,
            node_modules = package_node_modules,
            visibility = visibility,
        )

    _typescript_check(
        tsc = ":{}".format(tsc_bin),
        package_node_modules = package_node_modules,
        visibility = visibility,
        **kwargs,
    )

def typescript_dist(
        tsc = ":tsc",
        package_node_modules = ":node_modules",
        visibility = ["PUBLIC"],
        **kwargs):
    _typescript_dist(
        tsc = tsc,
        package_node_modules = package_node_modules,
        visibility = visibility,
        **kwargs,
    )

def vite_app(
        vite_bin = "vite",
        package_node_modules = ":node_modules",
        visibility = ["PUBLIC"],
        **kwargs):
    if not rule_exists(vite_bin):
        _npm_bin(
            name = vite_bin,
            node_modules = package_node_modules,
            visibility = visibility,
        )

    _vite_app(
        vite = ":{}".format(vite_bin),
        package_node_modules = package_node_modules,
        visibility = visibility,
        **kwargs,
    )

def workspace_node_modules(
        visibility = ["PUBLIC"],
        **kwargs):
    _workspace_node_modules(visibility = visibility, **kwargs)
