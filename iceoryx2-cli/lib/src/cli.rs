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

pub enum HelpOptions {
    DontPrintCommandSection,
    PrintCommandSection,
    PrintCommandSectionWithExternalCommandHint,
}

pub fn help_template(command_help: HelpOptions) -> String {
    let mut template = format!(
        "{{about}}\n\n\
         {} {{usage}}\n\n\
         {}\n\
         {{options}}",
        "Usage:".bright_green().bold(),
        "Options:".bright_green().bold(),
    );

    match command_help {
        HelpOptions::PrintCommandSection
        | HelpOptions::PrintCommandSectionWithExternalCommandHint => {
            template.push_str(&format!(
                "\n\n\
                {}\n\
                {{subcommands}}",
                "Commands:".bright_green().bold(),
            ));

            if let HelpOptions::PrintCommandSectionWithExternalCommandHint = command_help {
                template.push_str(&format!(
                    "\n\
                    {}{}",
                    "  ...            ".bold(),
                    "See external installed commands with --list"
                ));
            }
        }
        HelpOptions::DontPrintCommandSection => {}
    }

    template
}
