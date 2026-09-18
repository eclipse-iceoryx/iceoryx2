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

default_role = 'py:obj'

intersphinx_mapping = {'python': ('https://docs.python.org/3', None)}

# -- Options for HTML output -------------------------------------------------
# https://www.sphinx-doc.org/en/master/usage/configuration.html#options-for-html-output

html_theme = "furo"
html_title = "iceoryx2 <br><small style='font-size: 0.7em;'>Python Language Bindings</small>"

html_static_path = ['_static']
html_css_files = [
    'custom.css',
]


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
