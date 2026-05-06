// Copyright (c) 2024 Contributors to the Eclipse Foundation
//
// See the NOTICE file(s) distributed with this work for additional
// information regarding copyright ownership.
//
// This program and the accompanying materials are made available under the
// terms of the Apache Software License 2.0 which is available at
// https://www.apache.org/licenses/LICENSE-2.0, or the MIT license
// which is available at https://opensource.org/licenses/MIT.
//
// SPDX-License-Identifier: Apache-2.0 OR MIT

use colored::*;

#[derive(Default)]
pub struct HelpTemplate {
    has_positionals: bool,
    has_subcommands: bool,
    show_external_command_hint: bool,
}

pub fn help_template() -> HelpTemplate {
    HelpTemplate::default()
}

impl HelpTemplate {
    pub fn with_positionals(mut self) -> Self {
        self.has_positionals = true;
        self
    }

    pub fn with_subcommands(mut self) -> Self {
        self.has_subcommands = true;
        self
    }

    pub fn with_external_command_hint(mut self) -> Self {
        self.has_subcommands = true;
        self.show_external_command_hint = true;
        self
    }

    pub fn build(self) -> String {
        let mut template = format!(
            "{{about}}\n\n\
             {} {{usage}}\n\n",
            "Usage:".bright_green().bold(),
        );

        if self.has_positionals {
            template.push_str("{positionals}\n\n");
        }

        template.push_str(&format!(
            "{}\n\
             {{options}}",
            "Options:".bright_green().bold(),
        ));

        if self.has_subcommands {
            template.push_str(&format!(
                "\n\n\
                 {}\n\
                 {{subcommands}}",
                "Commands:".bright_green().bold(),
            ));

            if self.show_external_command_hint {
                template.push_str(&format!(
                    "\n\
                     {}{}",
                    "  ...            ".bold(),
                    "See external installed commands with --list",
                ));
            }
        }

        template
    }
}
