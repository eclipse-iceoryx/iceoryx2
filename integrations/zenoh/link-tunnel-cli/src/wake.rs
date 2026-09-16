// Copyright (c) 2026 Contributors to the Eclipse Foundation
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

use core::time::Duration;

use iceoryx2::node::Node;
use iceoryx2::port::listener::Listener;
use iceoryx2::prelude::*;
use iceoryx2::service::service_name::ServiceName;
use iceoryx2_link_backend::WakeService;
use iceoryx2_log::{fail, info, warn};

use crate::ORIGIN;
use crate::cli::Cli;
use crate::tunnel::ZenohTunnel;

const DEFAULT_POLL_RATE_MS: u64 = 100;

pub struct WakeSources {
    poll: Option<Duration>,
    tunnel: Option<Listener<WakeService>>,
    user: Vec<Listener<ipc::Service>>,
}

impl WakeSources {
    /// The wake sources the command line selects, polling by default and
    /// opt-in via `--poll` once another source is given.
    pub fn new(cli: &Cli, tunnel: &mut ZenohTunnel) -> anyhow::Result<Self> {
        let poll = match cli.poll {
            Some(rate) => Some(rate),
            None if !cli.reactive && cli.listener.is_empty() => Some(DEFAULT_POLL_RATE_MS),
            None => None,
        };
        match poll {
            Some(rate) => info!(from ORIGIN, "Polling at {}ms", rate),
            None => info!(from ORIGIN, "Polling disabled"),
        }
        let tunnel_listener = match cli.reactive {
            true => {
                let listener = fail!(
                    from ORIGIN,
                    when tunnel.listener(),
                    "Failed to create the tunnel's listener"
                );
                info!(from ORIGIN, "Reactive");
                Some(listener)
            }
            false => None,
        };
        let user = fail!(
            from ORIGIN,
            when open_user_listeners(tunnel.node(), &cli.listener),
            "Failed to open the --listener event services"
        );
        Ok(Self {
            poll: poll.map(Duration::from_millis),
            tunnel: tunnel_listener,
            user,
        })
    }

    /// Attaches every source to `waitset`, for as long as the guards live.
    pub fn attach<'w, 's>(
        &'s self,
        waitset: &'w WaitSet<ipc::Service>,
    ) -> anyhow::Result<Attached<'w, 's>> {
        let interval = match self.poll {
            Some(interval) => Some(fail!(
                from ORIGIN,
                when waitset.attach_interval(interval),
                "Failed to attach the polling interval"
            )),
            None => None,
        };
        let mut listeners = Vec::with_capacity(1 + self.user.len());
        if let Some(listener) = &self.tunnel {
            listeners.push(fail!(
                from ORIGIN,
                when waitset.attach_notification(listener),
                "Failed to attach the tunnel's listener"
            ));
        }
        for listener in &self.user {
            listeners.push(fail!(
                from ORIGIN,
                when waitset.attach_notification(listener),
                "Failed to attach a --listener"
            ));
        }
        Ok(Attached {
            _interval: interval,
            _listeners: listeners,
        })
    }

    /// Reads every pending notification, so a wake is consumed.
    pub fn drain(&self) {
        if let Some(listener) = &self.tunnel {
            drain(listener);
        }
        for listener in &self.user {
            drain(listener);
        }
    }
}

pub struct Attached<'w, 's> {
    _interval: Option<WaitSetGuard<'w, 'static, ipc::Service>>,
    _listeners: Vec<WaitSetGuard<'w, 's, ipc::Service>>,
}

fn open_user_listeners(
    node: &Node<ipc::Service>,
    names: &[String],
) -> anyhow::Result<Vec<Listener<ipc::Service>>> {
    let mut listeners = Vec::with_capacity(names.len());
    for name in names {
        let service_name = fail!(
            from ORIGIN,
            when ServiceName::new(name),
            "Invalid --listener service name {:?}", name
        );
        let service = fail!(
            from ORIGIN,
            when node.service_builder(&service_name).event().open_or_create(),
            "Failed to open the --listener event service {:?}", name
        );
        let listener = fail!(
            from ORIGIN,
            when service.listener_builder().create(),
            "Failed to create the listener on {:?}", name
        );
        info!(from ORIGIN, "Listener: {:?}", name);
        listeners.push(listener);
    }
    Ok(listeners)
}

fn drain<S: Service>(listener: &Listener<S>) {
    if let Err(error) = listener.try_wait(|_| {}) {
        warn!(from ORIGIN, "Draining a listener failed: {}", error);
    }
}
