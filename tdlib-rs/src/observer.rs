// Copyright 2020 - developers of the `grammers` project.
// Copyright 2021 - developers of the `tdlib-rs` project.
// Copyright 2024 - developers of the `tgt` project.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.
use futures_channel::oneshot;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::RwLock;

pub(super) struct Observer {
    requests: RwLock<HashMap<u32, oneshot::Sender<Value>>>,
}

impl Observer {
    pub fn new() -> Self {
        Self {
            requests: RwLock::default(),
        }
    }

    pub fn subscribe(&self, extra: u32) -> oneshot::Receiver<Value> {
        let (sender, receiver) = oneshot::channel();
        self.requests.write().unwrap().insert(extra, sender);
        receiver
    }

    pub fn notify(&self, response: Value) {
        let extra = response
            .get("@extra")
            .and_then(|v| v.as_u64())
            .and_then(|e| u32::try_from(e).ok());

        let extra = match extra {
            Some(id) => id,
            None => {
                log::error!("Response missing or invalid '@extra' field: {:?}", response);
                return;
            }
        };

        if let Some(sender) = self.requests.write().unwrap().remove(&extra) {
            if sender.send(response).is_err() {
                log::warn!("Got a response for a request that was dropped by receiver");
            }
        } else {
            log::warn!("Got a response for unknown request with extra {}", extra);
        }
    }
}