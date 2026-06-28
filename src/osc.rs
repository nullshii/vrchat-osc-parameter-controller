use std::time::Duration;

use vrchat_osc::VRChatOSC;

const RETRY_COUNT: u8 = 30;

pub async fn run_osc_loop() {
    let vrchat_osc = match VRChatOSC::new(None).await {
        Ok(v) => v,
        Err(e) => {
            log::error!("[OSC] Initialization error: {}", e);
            return;
        }
    };

    let cloned_vrchat_osc = vrchat_osc.clone();

    vrchat_osc
        .on_connect(move |res| match res {
            vrchat_osc::ServiceType::OscQuery(_name, addr) => {
                let vrchat_osc = cloned_vrchat_osc.clone();
                tokio::spawn(async move {
                    let mut counter = 0;
                    let params = loop {
                        if counter == RETRY_COUNT {
                            panic!("[OSC] Failed to get parameters after {} tries", RETRY_COUNT);
                        }
                        counter += 1;

                        match vrchat_osc
                            .get_parameter_from_addr("/avatar/parameters", addr)
                            .await
                        {
                            Ok(v) => break v,
                            Err(_) => {
                                tokio::time::sleep(Duration::from_secs(1)).await;
                            }
                        }
                    };
                    log::info!("Received parameters: {:?}", params);
                });
            }
            _ => {}
        })
        .await;
}
