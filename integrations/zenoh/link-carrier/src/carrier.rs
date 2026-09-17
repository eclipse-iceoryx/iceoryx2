// Copyright (c) 2025 Contributors to the Eclipse Foundation
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

use std::collections::BTreeMap;
use std::sync::{Arc, OnceLock};

use iceoryx2::service::service_hash::ServiceHash;
use iceoryx2_bb_elementary::generation::Generation;
use iceoryx2_link_backend::service_description::ServiceDescriptor;
use iceoryx2_link_backend::{Reactive, WakeHandle};
use iceoryx2_link_carrier::{Announcement, Carrier, Offer, PeerId};
use iceoryx2_log::{error, fail, origin, trace};
use zenoh::liveliness::LivelinessToken;
use zenoh::query::Queryable;
use zenoh::sample::Locality;
use zenoh::{Session, Wait};

use crate::channel::{ChannelError, ZenohChannel};
use crate::fingerprint::Encoded;
use crate::keys;
use crate::offers::OfferTracker;

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum CreationError {
    Session,
    Subscriber,
}

impl core::fmt::Display for CreationError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "CreationError::{self:?}")
    }
}

impl core::error::Error for CreationError {}

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum AnnouncementError {
    Encoding,
    Token,
    Queryable,
}

impl core::fmt::Display for AnnouncementError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "AnnouncementError::{self:?}")
    }
}

impl core::error::Error for AnnouncementError {}

/// One of this side's offers as declared on zenoh.
struct Announced {
    _token: LivelinessToken,
    _queryable: Queryable<()>,
}

/// A carrier over a zenoh session.
pub struct ZenohCarrier {
    session: Session,
    id: PeerId,
    /// This side's offers by service.
    announced: BTreeMap<ServiceHash, Announced>,
    /// The subscription to the peers' offers.
    _subscriber: zenoh::pubsub::Subscriber<()>,
    tracker: Arc<OfferTracker>,
    wake: Arc<OnceLock<WakeHandle>>,
}

impl ZenohCarrier {
    /// Creates a carrier on a session opened with `config`.
    pub fn create(config: zenoh::Config) -> Result<Self, CreationError> {
        let origin = origin!("ZenohCarrier::create");
        let session = fail!(
            from origin,
            when zenoh::open(config).wait(),
            with CreationError::Session,
            "Failed to open the zenoh session"
        );
        Self::open(session)
    }

    /// Creates a carrier on `session`.
    pub fn open(session: Session) -> Result<Self, CreationError> {
        let origin = origin!("ZenohCarrier::open");

        let id = PeerId::new(session.zid().to_le_bytes());
        let wake = Arc::new(OnceLock::new());
        let tracker = Arc::new(OfferTracker::new(wake.clone()));
        let subscriber = fail!(
            from origin,
            when session
                .liveliness()
                .declare_subscriber(keys::offers())
                .history(true)
                .callback({
                    let session = session.clone();
                    let tracker = tracker.clone();
                    move |sample| tracker.on_liveliness(&session, &id, sample)
                })
                .wait(),
            with CreationError::Subscriber,
            "Failed to subscribe to the peers' offers"
        );
        Ok(Self {
            session,
            id,
            announced: BTreeMap::new(),
            _subscriber: subscriber,
            tracker,
            wake,
        })
    }

    /// The id this carrier is known to the peers as.
    pub fn id(&self) -> PeerId {
        self.id
    }
}

impl Carrier for ZenohCarrier {
    type AnnouncementError = AnnouncementError;
    type ListError = core::convert::Infallible;
    type ChannelError = ChannelError;
    type Channel = ZenohChannel;

    fn announce(&mut self, announcement: Announcement) -> Result<(), Self::AnnouncementError> {
        let origin = origin!("ZenohCarrier::announce");
        match announcement {
            Announcement::Offered { descriptor } => {
                let encoded = fail!(
                    from origin,
                    when Encoded::encode(&descriptor),
                    with AnnouncementError::Encoding,
                    "Failed to encode the descriptor of {}", descriptor.name
                );
                let key = keys::offer(&descriptor.hash, encoded.fingerprint(), &self.id);
                let reply_key = key.clone();
                let bytes = encoded.bytes().to_vec();
                let queryable = fail!(
                    from origin,
                    when self
                        .session
                        .declare_queryable(key.clone())
                        .callback(move |query| {
                            if let Err(error) = query.reply(reply_key.clone(), bytes.clone()).wait() {
                                error!(from origin, "Failed to reply with the descriptor at {}: {}", reply_key, error);
                            }
                        })
                        .allowed_origin(Locality::Remote)
                        .wait(),
                    with AnnouncementError::Queryable,
                    "Failed to declare the queryable of {}", descriptor.name
                );
                let token = fail!(
                    from origin,
                    when self.session.liveliness().declare_token(key.clone()).wait(),
                    with AnnouncementError::Token,
                    "Failed to declare the liveliness token of {}", descriptor.name
                );
                trace!(from origin, "Declared the offer of {} at {}", descriptor.name, key);
                self.announced.insert(
                    descriptor.hash,
                    Announced {
                        _token: token,
                        _queryable: queryable,
                    },
                );
            }
            Announcement::Withdrawn { hash } => {
                trace!(from origin, "Undeclared the offer of the service {}", hash.as_str());
                self.announced.remove(&hash);
            }
        }
        Ok(())
    }

    fn generation(&self) -> Generation {
        self.tracker.generation()
    }

    fn offers(&self, callback: &mut dyn FnMut(Offer)) -> Result<(), Self::ListError> {
        self.tracker.table().each(callback);
        Ok(())
    }

    fn open_channel(
        &mut self,
        descriptor: &ServiceDescriptor,
    ) -> Result<Self::Channel, Self::ChannelError> {
        let origin = origin!("ZenohCarrier::open_channel");

        let encoded = fail!(
            from origin,
            when Encoded::encode(descriptor),
            with ChannelError::Publisher,
            "Failed to encode the descriptor of {}", descriptor.name
        );
        let key = keys::channel(&descriptor.hash, encoded.fingerprint());
        ZenohChannel::open(&self.session, key, self.wake.clone())
    }
}

impl Reactive for ZenohCarrier {
    fn attach(&mut self, wake: WakeHandle) {
        let _ = self.wake.set(wake);
    }
}
