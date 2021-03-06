// btleplug Source Code File
//
// Copyright 2020 Nonpolynomial Labs LLC. All rights reserved.
//
// Licensed under the BSD 3-Clause license. See LICENSE file in the project root
// for full license information.
//
// Some portions of this file are taken and/or modified from Rumble
// (https://github.com/mwylde/rumble), using a dual MIT/Apache License under the
// following copyright:
//
// Copyright (c) 2014 The Rust Project Developers

use crate::{
    api::{
        Central, CentralEvent, BDAddr, AdapterManager
    },
    Result
};
use super::{
    peripheral::Peripheral,
    ble::watcher::BLEWatcher,
    utils,
};
use std::sync::{mpsc::Receiver, Arc, Mutex};

#[derive(Clone)]
pub struct Adapter {
    watcher: Arc<Mutex<BLEWatcher>>,
    manager: AdapterManager<Peripheral>
}

impl Adapter {
    pub fn new() -> Self {
        let watcher = Arc::new(Mutex::new(BLEWatcher::new()));
        let manager = AdapterManager::<Peripheral>::new();
        Adapter { watcher, manager }
    }
}

impl Central<Peripheral> for Adapter {
    fn event_receiver(&self) -> Option<Receiver<CentralEvent>> {
        self.manager.event_receiver()
    }

    fn start_scan(&self) -> Result<()> {
        let watcher = self.watcher.lock().unwrap();
        let manager = self.manager.clone();
        watcher.start(Box::new(move |args| {
            let bluetooth_address = args.get_bluetooth_address().unwrap();
            let address = utils::to_addr(bluetooth_address);
            let peripheral = Peripheral::new(manager.clone(), address);
            peripheral.update_properties(&args);
            if !manager.has_peripheral(&address) {
                manager.add_peripheral(address, peripheral);
                manager.emit(CentralEvent::DeviceDiscovered(address));
            } else {
                manager.update_peripheral(address, peripheral);
                manager.emit(CentralEvent::DeviceUpdated(address));
            }
        }))
    }

    fn stop_scan(&self) -> Result<()> {
        let watcher = self.watcher.lock().unwrap();
        watcher.stop().unwrap();
        Ok(())
    }

    fn peripherals(&self) -> Vec<Peripheral> {
        self.manager.peripherals()
    }

    fn peripheral(&self, address: BDAddr) -> Option<Peripheral> {
        self.manager.peripheral(address)
    }

    fn active(&self, _enabled: bool) {
    }

    fn filter_duplicates(&self, _enabled: bool) {
    }
}
