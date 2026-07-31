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
    out = ctx.actions.declare_file(ctx.attr.output)

    substitutions = {}
    for key, value in ctx.attr.substitutions.items():
        substitutions["@{}@".format(key)] = value

    ctx.actions.expand_template(
        template = ctx.file.template,
        output = out,
        substitutions = substitutions,
    )

    return [DefaultInfo(files = depset([out]))]

configure_file = rule(
    implementation = _configure_file_impl,
    attrs = {
        "template": attr.label(
            allow_single_file = True,
            mandatory = True,
        ),
        "output": attr.string(
            mandatory = True,
        ),
        "substitutions": attr.string_dict(
            mandatory = True,
        ),
    },
)

