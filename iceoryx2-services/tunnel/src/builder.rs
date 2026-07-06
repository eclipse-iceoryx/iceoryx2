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

//! Type-state builder for the [`Tunnel`].
//!
//! [`TunnelBuilder`] selects between polled and reactive operation. The
//! reactive path is only available when the chosen [`Backend`]'s builder
//! implements [`ReactiveBackendBuilder`].
//!
//! Configurations are optional; any that are not supplied fall back to
//! [`Default::default()`].

use core::fmt::Debug;
use core::marker::PhantomData;

use alloc::boxed::Box;
use alloc::format;

use iceoryx2::node::NodeBuilder;
use iceoryx2::port::listener::Listener;
use iceoryx2::service::Service;
use iceoryx2_log::{fail, info, trace};
use iceoryx2_services_tunnel_backend::traits::{Backend, BackendBuilder, ReactiveBackendBuilder};
use iceoryx2_services_tunnel_backend::types::wake::WakeHandle;

use crate::discovery::LocalDiscoveryStrategy;
use crate::discovery::subscriber::DiscoverySubscriber;
use crate::discovery::tracker::DiscoveryTracker;
use crate::tunnel::{Config, CreationError, Tunnel};

/// Marker for a [`TunnelBuilder`] whose mode has not yet been chosen.
#[derive(Debug)]
pub struct Unconfigured;

/// Marker for a [`TunnelBuilder`] in polled mode.
#[derive(Debug)]
pub struct Polled;

/// Marker for a [`TunnelBuilder`] in reactive mode.
#[derive(Debug)]
pub struct Reactive;

/// Builder for [`Tunnel`].
///
/// Constructed via [`TunnelBuilder::new`] in the [`Unconfigured`] state. The
/// builder must transition through [`TunnelBuilder::polled`] or
/// [`TunnelBuilder::reactive`] before calling `create()`.
///
/// Any configuration that is not explicitly provided defaults to
/// [`Default::default()`].
pub struct TunnelBuilder<S, B, M = Unconfigured>
where
    S: Service,
    B: for<'b> Backend<S> + Debug,
{
    tunnel_config: Option<Config>,
    iceoryx_config: Option<iceoryx2::config::Config>,
    backend_config: Option<<B as Backend<S>>::Config>,
    translator: Option<<B as Backend<S>>::Translator>,
    mapping: Option<<B as Backend<S>>::Mapping>,
    _phantom: PhantomData<(S, B, M)>,
}

impl<S, B, M> TunnelBuilder<S, B, M>
where
    S: Service,
    B: for<'b> Backend<S> + Debug,
{
    /// Sets the [`Config`] for the [`Tunnel`].
    pub fn tunnel_config(mut self, config: Config) -> Self {
        self.tunnel_config = Some(config);
        self
    }

    /// Sets the [`iceoryx2::config::Config`] used for the tunnel's iceoryx
    /// services.
    pub fn iceoryx_config(mut self, config: iceoryx2::config::Config) -> Self {
        self.iceoryx_config = Some(config);
        self
    }

    /// Sets the [`Backend`]'s configuration.
    pub fn backend_config(mut self, config: <B as Backend<S>>::Config) -> Self {
        self.backend_config = Some(config);
        self
    }

    /// Sets the payload translation strategy applied by the backend. Defaults
    /// to [`Passthrough`](iceoryx2_services_tunnel_backend::traits::Passthrough)
    /// when not set.
    pub fn translator(mut self, translator: <B as Backend<S>>::Translator) -> Self {
        self.translator = Some(translator);
        self
    }

    /// Sets the name and QoS mapping strategy applied by the backend. Falls
    /// back to the backend's default strategy when not set.
    pub fn mapping(mut self, mapping: <B as Backend<S>>::Mapping) -> Self {
        self.mapping = Some(mapping);
        self
    }
}

impl<S, B> TunnelBuilder<S, B, Unconfigured>
where
    S: Service,
    B: for<'b> Backend<S> + Debug,
{
    /// Creates a new builder. All configurations default to
    /// [`Default::default()`] unless overridden via the chained setters.
    pub fn new() -> Self {
        Self {
            tunnel_config: None,
            iceoryx_config: None,
            backend_config: None,
            translator: None,
            mapping: None,
            _phantom: PhantomData,
        }
    }

    /// Selects polled mode. The resulting tunnel must be driven by repeated
    /// calls to [`Tunnel::discover`] and [`Tunnel::propagate`].
    pub fn polled(self) -> TunnelBuilder<S, B, Polled> {
        TunnelBuilder {
            tunnel_config: self.tunnel_config,
            iceoryx_config: self.iceoryx_config,
            backend_config: self.backend_config,
            translator: self.translator,
            mapping: self.mapping,
            _phantom: PhantomData,
        }
    }
}

impl<S, B> TunnelBuilder<S, B, Unconfigured>
where
    S: Service,
    B: for<'b> Backend<S> + Debug,
    for<'b> B::Builder<'b>: ReactiveBackendBuilder<S>,
{
    /// Selects reactive mode. Only available when the chosen [`Backend`]'s
    /// builder implements [`ReactiveBackendBuilder`].
    pub fn reactive(self) -> TunnelBuilder<S, B, Reactive> {
        TunnelBuilder {
            tunnel_config: self.tunnel_config,
            iceoryx_config: self.iceoryx_config,
            backend_config: self.backend_config,
            translator: self.translator,
            mapping: self.mapping,
            _phantom: PhantomData,
        }
    }
}

impl<S, B> Default for TunnelBuilder<S, B, Unconfigured>
where
    S: Service,
    B: for<'b> Backend<S> + Debug,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<S, B> TunnelBuilder<S, B, Polled>
where
    S: Service,
    B: for<'b> Backend<S> + Debug,
{
    /// Builds the tunnel in polled mode.
    pub fn create(self) -> Result<Tunnel<S, B>, CreationError> {
        create_polled::<S, B>(
            self.tunnel_config.unwrap_or_default(),
            self.iceoryx_config.unwrap_or_default(),
            self.backend_config.unwrap_or_default(),
            self.translator.unwrap_or_default(),
            self.mapping.unwrap_or_default(),
        )
    }
}

impl<S, B> TunnelBuilder<S, B, Reactive>
where
    S: Service,
    B: for<'b> Backend<S> + Debug,
    for<'b> B::Builder<'b>: ReactiveBackendBuilder<S>,
{
    /// Builds the tunnel in reactive mode.
    pub fn create<W>(self) -> Result<(Tunnel<S, B>, Listener<W>), CreationError>
    where
        W: Service,
        for<'b> B::Builder<'b>: ReactiveBackendBuilder<S, WakeService = W>,
    {
        create_reactive::<S, B, W>(
            self.tunnel_config.unwrap_or_default(),
            self.iceoryx_config.unwrap_or_default(),
            self.backend_config.unwrap_or_default(),
            self.translator.unwrap_or_default(),
            self.mapping.unwrap_or_default(),
        )
    }
}

/// Builds a tunnel in polled mode.
fn create_polled<S, B>(
    tunnel_config: Config,
    iceoryx_config: iceoryx2::config::Config,
    backend_config: <B as Backend<S>>::Config,
    translator: <B as Backend<S>>::Translator,
    mapping: <B as Backend<S>>::Mapping,
) -> Result<Tunnel<S, B>, CreationError>
where
    S: Service,
    B: for<'a> Backend<S> + Debug,
{
    let origin = "TunnelBuilder::<Polled>::create";

    let backend = fail!(
        from origin,
        when B::builder(&backend_config)
            .translator(translator)
            .mapping(mapping)
            .create(),
        with CreationError::Backend,
        "Failed to create backend"
    );

    create(tunnel_config, iceoryx_config, backend)
}

/// Builds a tunnel in reactive mode.
fn create_reactive<S, B, W>(
    tunnel_config: Config,
    iceoryx_config: iceoryx2::config::Config,
    backend_config: <B as Backend<S>>::Config,
    translator: <B as Backend<S>>::Translator,
    mapping: <B as Backend<S>>::Mapping,
) -> Result<(Tunnel<S, B>, Listener<W>), CreationError>
where
    S: Service,
    W: Service,
    B: for<'a> Backend<S> + Debug,
    for<'b> B::Builder<'b>: ReactiveBackendBuilder<S, WakeService = W>,
{
    let origin = "TunnelBuilder::<Reactive>::create";

    let reactive_node = fail!(
        from origin,
        when NodeBuilder::new().create::<W>(),
        with CreationError::ReactiveMode,
        "Failed to create local node for wake event service"
    );

    let service_name_str = format!("iox2://tunnel/event/{}", reactive_node.id());
    let service_name = fail!(
        from origin,
        when service_name_str.as_str().try_into(),
        with CreationError::ServiceName,
        "Failed to create wake service name"
    );

    let wake_service = fail!(
        from origin,
        when reactive_node.service_builder(&service_name).event().create(),
        with CreationError::ReactiveMode,
        "Failed to open wake event service"
    );

    let listener = fail!(
        from origin,
        when wake_service.listener_builder().create(),
        with CreationError::ReactiveMode,
        "Failed to create wake listener"
    );

    let notifier = fail!(
        from origin,
        when wake_service.notifier_builder().create(),
        with CreationError::ReactiveMode,
        "Failed to create wake notifier"
    );

    let wake = WakeHandle::new(notifier);

    // The reactive_node is no longer needed after the wake event service's
    // listener and notifier exist — iceoryx2 keeps the underlying service
    // alive via port refcounts.
    drop(reactive_node);

    let backend = fail!(
        from origin,
        when B::builder(&backend_config)
            .translator(translator)
            .mapping(mapping)
            .reactive(wake)
            .create(),
        with CreationError::Backend,
        "Failed to create reactive backend"
    );

    let tunnel = create(tunnel_config, iceoryx_config, backend)?;

    Ok((tunnel, listener))
}

/// Finalizes the build of the Tunnel.
fn create<S, B>(
    tunnel_config: Config,
    iceoryx_config: iceoryx2::config::Config,
    backend: B,
) -> Result<Tunnel<S, B>, CreationError>
where
    S: Service,
    B: for<'a> Backend<S> + Debug,
{
    let origin = format!(
        "TunnelBuilder<{}, {}>::create",
        core::any::type_name::<S>(),
        core::any::type_name::<B>()
    );

    trace!(
        from origin,
        "Building Tunnel:\n{:?}\n{:?}",
        &tunnel_config, &iceoryx_config
    );

    let tunnel_node = fail!(
        from origin,
        when NodeBuilder::new().config(&iceoryx_config).create::<S>(),
        with CreationError::Node,
        "Failed to create Tunnel node"
    );

    let local_discovery = match &tunnel_config.discovery_service {
        Some(service_name) => {
            info!(from origin, "Local Discovery via Subscriber");

            let service_name = fail!(
                from origin,
                when service_name.as_str().try_into(),
                with CreationError::ServiceName,
                "Failed to create service name {}", service_name
            );

            let subscriber = fail!(
                from origin,
                when DiscoverySubscriber::create(&tunnel_node, service_name),
                with CreationError::DiscoverySubscriber,
                "Failed to create discovery subscriber"
            );

            LocalDiscoveryStrategy::Subscriber(subscriber)
        }
        None => {
            info!(from origin, "Local Discovery via Tracker");
            let tracker = DiscoveryTracker::create(&iceoryx_config);
            LocalDiscoveryStrategy::Tracker(Box::new(tracker))
        }
    };

    let services_filter = tunnel_config
        .services
        .as_ref()
        .map(|names| names.iter().cloned().collect());

    Ok(Tunnel::create(
        tunnel_node,
        backend,
        local_discovery,
        services_filter,
    ))
}
