# Copyright (c) 2025 Contributors to the Eclipse Foundation
#
# See the NOTICE file(s) distributed with this work for additional
# information regarding copyright ownership.
#
# This program and the accompanying materials are made available under the
# terms of the Apache Software License 2.0 which is available at
# https://www.apache.org/licenses/LICENSE-2.0, or the MIT license
# which is available at https://opensource.org/licenses/MIT.
#
# SPDX-License-Identifier: Apache-2.0 OR MIT

# -- Project information -----------------------------------------------------
# https://www.sphinx-doc.org/en/master/usage/configuration.html#project-information

project = 'iceoryx2'
author = 'Contributors of eclipse-iceoryx'

# -- General configuration ---------------------------------------------------
# https://www.sphinx-doc.org/en/master/usage/configuration.html#general-configuration

extensions = [
    'sphinx.ext.autodoc',
    'sphinx.ext.napoleon',
    'sphinx.ext.viewcode',
    'sphinx.ext.intersphinx',
]

# a name in single backticks is a link to the python object of that name
default_role = 'py:obj'

# references to the python standard library link to the python documentation
intersphinx_mapping = {'python': ('https://docs.python.org/3', None)}

# a reference that does not resolve is a warning and fails the build
nitpicky = True
nitpick_ignore_regex = [
    # type variables in the signatures of the extension modules
    (r'py:(class|obj)', r'iceoryx2\.\w+\.(K|V|T|ReqT|ResT)'),
    # flatbuffers does not publish an inventory to link against
    (r'py:class', r'flatbuffers\.builder\.Builder'),
]

# the navigation lists members without their class prefix
toc_object_entries_show_parents = 'hide'

# -- Options for HTML output -------------------------------------------------
# https://www.sphinx-doc.org/en/master/usage/configuration.html#options-for-html-output

html_theme = "furo"
html_title = (
    "<span class='ix-brand__text'>"
    "<span class='ix-brand__word'>iceoryx<span class='ix-brand__two'>2</span></span> "
    "<span class='ix-brand__tag'>Python Bindings</span>"
    "</span>"
)

html_static_path = ['_static']
html_css_files = [
    'theme.css',
    'custom.css',
]
html_js_files = [
    'theme-transitions.js',
]

html_theme_options = {
    "top_of_page_buttons": [],
    "light_css_variables": {
        # --- iceoryx brand tokens (light) ---
        "ix-bg": "#f5f8fc",
        "ix-bg-2": "#e2eaf3",
        "ix-surface": "#fafcff",
        "ix-surface-2": "#f2f7fc",
        "ix-line": "rgba(15, 55, 90, 0.12)",
        "ix-line-soft": "rgba(15, 55, 90, 0.06)",
        "ix-box-line": "rgba(15, 55, 90, 0.28)",
        "ix-ice": "#0a8aa0",
        "ix-ice-bright": "#0a93ab",
        "ix-ice-deep": "#0a7385",
        "ix-blue": "#3a66cf",
        "ix-mint": "#0e9b73",
        "ix-text": "#0c1825",
        "ix-muted": "#36475a",
        "ix-muted-2": "#586a80",
        "ix-status-funding": "#708197",
        "ix-glow-1": "rgba(10, 160, 185, 0.16)",
        "ix-glow": "0 0 40px rgba(10, 138, 160, 0.25)",
        "ix-drawer-bg": "rgba(238, 243, 249, 0.72)",
        "ix-header-bg": "rgba(247, 250, 253, 0.82)",
        "ix-shadow": "0 16px 40px -30px rgba(20, 50, 90, 0.25)",
        "ix-card-bg": "#fafcff",
        "ix-card-shadow": "0 22px 44px -26px rgba(20, 50, 90, 0.28)",
        "ix-card-hover-border": "rgba(10, 138, 160, 0.40)",
        "ix-svg-panel": "transparent",
        "ix-svg-pad": "0",
        "ix-code-bg": "#eef3f9",
        "ix-code-hl": "rgba(10, 138, 160, 0.12)",
        "ix-selection-bg": "rgba(10, 138, 160, 0.22)",
        "ix-selection-fg": "#04222a",
        # --- Furo mappings (light) ---
        "color-brand-primary": "#0a8aa0",
        "color-brand-content": "#0a7d91",
        "color-background-primary": "#f5f8fc",
        "color-background-secondary": "#eef3f9",
        "color-background-hover": "#ffffff",
        "color-background-hover--transparent": "rgba(255,255,255,0)",
        "color-background-border": "rgba(15, 55, 90, 0.12)",
        "color-foreground-primary": "#0c1825",
        "color-foreground-secondary": "#46596b",
        "color-foreground-muted": "#586a80",
        "color-foreground-border": "rgba(15, 55, 90, 0.12)",
        "color-code-background": "#ffffff",
        "color-code-foreground": "#33485a",
        "color-link": "#0a7d91",
        "color-link--hover": "#0a93ab",
        "color-link-underline": "rgba(10, 138, 160, 0.30)",
        "color-link-underline--hover": "rgba(10, 138, 160, 0.7)",
        "color-sidebar-background": "#eef3f9",
        "color-sidebar-background-border": "rgba(15, 55, 90, 0.10)",
        "color-sidebar-item-background--hover": "rgba(10, 138, 160, 0.07)",
        "color-sidebar-link-text": "#46596b",
        "color-sidebar-link-text--top-level": "#0c1825",
        "color-sidebar-caption-text": "#586a80",
        "color-sidebar-search-border": "rgba(15, 55, 90, 0.12)",
        "color-toc-item-text": "#46596b",
        "color-toc-item-text--active": "#0a8aa0",
        "color-admonition-background": "#fafcff",
        "color-highlight-on-target": "rgba(10, 147, 171, 0.14)",
        "color-api-name": "var(--color-brand-content)",
        "color-api-pre-name": "var(--color-foreground-muted)",
    },
    "dark_css_variables": {
        # --- iceoryx brand tokens (dark) ---
        "ix-bg": "#060910",
        "ix-bg-2": "#080d16",
        "ix-surface": "#141d2d",
        "ix-surface-2": "#1b2740",
        "ix-line": "rgba(150, 200, 230, 0.16)",
        "ix-line-soft": "rgba(150, 200, 230, 0.06)",
        "ix-box-line": "var(--ix-line)",
        "ix-ice": "#2dd4e8",
        "ix-ice-bright": "#7ff0ff",
        "ix-ice-deep": "#16b8d4",
        "ix-blue": "#5b8def",
        "ix-mint": "#6ff0c7",
        "ix-text": "#eaf3f7",
        "ix-muted": "#bccbd6",
        "ix-muted-2": "#6b7d8c",
        "ix-status-funding": "#6b7d8c",
        "ix-glow-1": "rgba(45, 212, 232, 0.30)",
        "ix-glow": "0 0 40px rgba(45, 212, 232, 0.35)",
        "ix-drawer-bg": "rgba(8, 13, 22, 0.60)",
        "ix-header-bg": "rgba(8, 13, 22, 0.82)",
        "ix-shadow": "0 18px 50px -34px rgba(0, 0, 0, 0.80)",
        "ix-card-bg": "linear-gradient(180deg, #18222f, #121b29)",
        "ix-card-shadow": "0 24px 50px -28px rgba(0, 0, 0, 0.85)",
        "ix-card-hover-border": "rgba(45, 212, 232, 0.40)",
        "ix-svg-panel": "#f7fbfd",
        "ix-svg-pad": "1rem",
        "ix-code-bg": "#0a111c",
        "ix-code-hl": "rgba(45, 212, 232, 0.12)",
        "ix-selection-bg": "rgba(45, 212, 232, 0.28)",
        "ix-selection-fg": "#ffffff",
        # --- Furo mappings (dark) ---
        "color-brand-primary": "#2dd4e8",
        "color-brand-content": "#2dd4e8",
        "color-background-primary": "#060910",
        "color-background-secondary": "#080d16",
        "color-background-hover": "#0c1320",
        "color-background-hover--transparent": "rgba(12,19,32,0)",
        "color-background-border": "rgba(150, 200, 230, 0.10)",
        "color-foreground-primary": "#eaf3f7",
        "color-foreground-secondary": "#9fb1bf",
        "color-foreground-muted": "#6b7d8c",
        "color-foreground-border": "rgba(150, 200, 230, 0.10)",
        "color-code-background": "#0a111c",
        "color-code-foreground": "#c5d4e0",
        "color-link": "#2dd4e8",
        "color-link--hover": "#7ff0ff",
        "color-link-underline": "rgba(45, 212, 232, 0.28)",
        "color-link-underline--hover": "rgba(127, 240, 255, 0.8)",
        "color-sidebar-background": "#080d16",
        "color-sidebar-background-border": "rgba(150, 200, 230, 0.08)",
        "color-sidebar-item-background--hover": "rgba(45, 212, 232, 0.07)",
        "color-sidebar-link-text": "#9fb1bf",
        "color-sidebar-link-text--top-level": "#eaf3f7",
        "color-sidebar-caption-text": "#6b7d8c",
        "color-sidebar-search-border": "rgba(150, 200, 230, 0.12)",
        "color-toc-item-text": "#9fb1bf",
        "color-toc-item-text--active": "#2dd4e8",
        "color-admonition-background": "#0c1320",
        "color-highlight-on-target": "rgba(45, 212, 232, 0.16)",
        "color-api-name": "var(--color-brand-content)",
        "color-api-pre-name": "var(--color-foreground-muted)",
    },
}


# -- Cross references --------------------------------------------------------

def resolve_in_package(app, env, node, contnode):
    """Resolves a reference from a submodule docstring relative to the package."""
    if node.get('refdomain') != 'py' or node.get('py:module') == 'iceoryx2':
        return None

    node['py:module'] = 'iceoryx2'
    node['py:class'] = None
    return env.get_domain('py').resolve_xref(
        env, node['refdoc'], app.builder, node['reftype'], node['reftarget'], node, contnode
    )


def setup(app):
    app.connect('missing-reference', resolve_in_package)
