# Copyright (c) 2026 Contributors to the Eclipse Foundation
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

"""
configure_file like with cmake
"""

def _configure_file_impl(ctx):
    substitutions = {}

    key_value_pairs = ctx.attr.substitutions
    if len(key_value_pairs) % 2 != 0:
        fail("Substitutions must contain key/value pairs")

    for i in range(0, len(key_value_pairs), 2):
        key = key_value_pairs[i]
        value = key_value_pairs[i + 1]
        substitutions["@" + key + "@"] = value

    ctx.actions.expand_template(
        template = ctx.file.template,
        output = ctx.outputs.out,
        substitutions = substitutions,
    )

# Expands a template file with configurable substitutions
#
# Each substitution is specified as a pair of strings:
#
#     ["KEY", "VALUE"]
#
# Multiple substitutions can be combined by concatenating concatination with "+".
#
# Config options can be used with select() expressions:
#
#     substitutions = ["FOO_ENABLED"] + select({ "//:cfg_foo": ["1"], "//conditions:default": ["0"] })
#                     + ["BAR_ENABLED"] + select({ "//:cfg_bar": ["1"], "//conditions:default": ["0"] })
#
# The rule wraps "KEY" into "@" and substitutes it with the "VALUE" in the template, e.g.
# "FOO_ENABLED" becomes "@FOO_ENABLED@" and the "@FOO_ENABLED@" from the template will be replaces by "VALUE"
#
configure_file = rule(
    implementation = _configure_file_impl,
    attrs = {
        "template": attr.label(
            allow_single_file = True,
            mandatory = True,
        ),
        "out": attr.output(
            mandatory = True,
        ),
        "substitutions": attr.string_list(
            mandatory = True,
        ),
    },
)
