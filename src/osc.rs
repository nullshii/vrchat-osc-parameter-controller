use std::{
    net::SocketAddr,
    sync::{Arc, Mutex},
    time::Duration,
};

use rosc::OscPacket;
use tokio::sync::mpsc::{Receiver, Sender};
use vrchat_osc::{
    VRChatOSC,
    models::{OscNode, OscRootNode},
};

const RETRY_COUNT: u8 = 30;

#[derive(Debug)]
pub enum OscNotification {
    AvatarParametersUpdated {
        address: SocketAddr,
        parameters: OscNode,
    },
    PacketReceived {
        address: SocketAddr,
        packet: OscPacket,
    },
    SendUpdatedParameter {
        packet: OscPacket,
    },
}

pub async fn run_osc_loop(tx: Sender<OscNotification>, mut rx: Receiver<OscNotification>) {
    let vrchat_osc = match VRChatOSC::new(None).await {
        Ok(v) => v,
        Err(e) => {
            log::error!("[OSC] Initialization error: {}", e);
            return;
        }
    };

    // Use Arc<Mutex<...>> so both event loops can safely access/mutate the address
    let address_shared = Arc::new(Mutex::new(None::<SocketAddr>));

    // Clone specifically for the on_connect block
    let vrchat_osc_for_connect = vrchat_osc.clone();
    let address_for_connect = address_shared.clone();
    let tx_for_connect = tx.clone();

    vrchat_osc
        .on_connect(move |res| match res {
            vrchat_osc::ServiceType::OscQuery(_name, addr) => {
                let vrchat_osc_task = vrchat_osc_for_connect.clone();
                let address_task = address_for_connect.clone();
                let tx_task = tx_for_connect.clone();

                tokio::spawn(async move {
                    let mut counter = 0;
                    let params = loop {
                        if counter == RETRY_COUNT {
                            panic!("[OSC] Failed to get parameters after {} tries", RETRY_COUNT);
                        }
                        counter += 1;

                        match vrchat_osc_task
                            .get_parameter_from_addr("/avatar/parameters", addr)
                            .await
                        {
                            Ok(v) => {
                                if let Ok(mut guard) = address_task.lock() {
                                    *guard = Some(addr);
                                }

                                break v;
                            }
                            Err(_) => {
                                tokio::time::sleep(Duration::from_secs(1)).await;
                            }
                        }
                    };
                    let _ = tx_task
                        .send(OscNotification::AvatarParametersUpdated {
                            address: addr,
                            parameters: params,
                        })
                        .await;
                });
            }
            _ => {}
        })
        .await;

    let root_node = OscRootNode::new().with_avatar();

    // Clone specifically for the register block
    let vrchat_osc_for_register = vrchat_osc.clone();
    let address_for_register = address_shared.clone();
    let tx_for_register = tx.clone();

    match vrchat_osc
        .register("HeartRate-Service", root_node, move |packet| {
            let current_addr = address_for_register.lock().ok().and_then(|guard| *guard);

            if let OscPacket::Message(ref msg) = packet
                && let Some(addr) = current_addr
            {
                let vrchat_osc_task = vrchat_osc_for_register.clone();
                let tx_task = tx_for_register.clone();

                if msg.addr == "/avatar/change" {
                    tokio::spawn(async move {
                        let mut counter = 0;
                        let params = loop {
                            if counter == RETRY_COUNT {
                                panic!(
                                    "[OSC] Failed to get parameters after {} tries",
                                    RETRY_COUNT
                                );
                            }
                            counter += 1;

                            match vrchat_osc_task
                                .get_parameter_from_addr("/avatar/parameters", addr)
                                .await
                            {
                                Ok(v) => break v,
                                Err(_) => {
                                    tokio::time::sleep(Duration::from_secs(1)).await;
                                }
                            }
                        };
                        let _ = tx_task
                            .send(OscNotification::AvatarParametersUpdated {
                                address: addr,
                                parameters: params,
                            })
                            .await;
                    });
                } else {
                    tokio::spawn(async move {
                        let _ = tx_task
                            .send(OscNotification::PacketReceived {
                                address: addr,
                                packet,
                            })
                            .await;
                    });
                }
            }
        })
        .await
    {
        Ok(_) => log::info!("[OSC] service registered"),
        Err(e) => log::warn!("[OSC] Failed to register client: {}", e),
    }

    while let Some(notification) = rx.recv().await {
        match notification {
            OscNotification::SendUpdatedParameter { packet } => {
                match vrchat_osc.send(packet, "VRChat-Client-*").await {
                    Ok(_) => {}
                    Err(e) => {
                        log::error!("[OSC] Failed to send OSC message: {}", e);
                    }
                };
            }
            _ => {}
        }
    }
}
