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

use iceoryx2_bb_print::*;

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
             {GREEN}{BOLD}Usage: {{usage}}{RESET}\n\n"
        );

        if self.has_positionals {
            template.push_str("{positionals}\n\n");
        }

        template.push_str(&format!(
            "{BRIGHT_GREEN}{BOLD}Options:{RESET}\n\
             {BOLD}{{options}}{RESET}",
        ));

        if self.has_subcommands {
            template.push_str(&format!(
                "\n\n\
                 {BRIGHT_GREEN}{BOLD}Commands:{RESET}\n\
                 {BOLD}{{subcommands}}{RESET}"
            ));

            if self.show_external_command_hint {
                template.push_str(&format!(
                    "\n\
                     {BOLD}{}{RESET}{}",
                    "  ...            ", "See external installed commands with --list",
                ));
            }
        }

        template
    }
}
