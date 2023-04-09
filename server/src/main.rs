use std::{
    error::Error,
    sync::Arc
};
use tokio::sync::Mutex;
use rtmp::{
    channels::{
        channels::ChannelsManager,
        define::{ClientEvent,ChannelData}
    },
    session::{
        common::SessionInfo, 
        define::SessionSubType
    },
    rtmp::RtmpServer
};
use client::client::RtmpClient;
use log::LevelFilter;
use log4rs::{
    append::{
        console::ConsoleAppender,
        file::FileAppender
    },
    encode::pattern::PatternEncoder,
    config::{Appender, Config, Root},
    filter::threshold::ThresholdFilter
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let _stdout = ConsoleAppender::builder()
        .encoder(Box::new(PatternEncoder::new("{d(%Y-%m-%d %H:%M:%S)} {l} {t} - {m}{n}")))
        .build();

    let _logfile = FileAppender::builder()
        .encoder(Box::new(PatternEncoder::new("{d(%Y-%m-%d %H:%M:%S)} {l} {t} - {m}{n}")))
        .build("log/rtmp.log")
        .unwrap();

    let config = Config::builder()
        .appender(Appender::builder()
            .filter(Box::new(ThresholdFilter::new(LevelFilter::Info)))
            .build("stdout", Box::new(_stdout))
        )
        .appender(Appender::builder()
            .filter(Box::new(ThresholdFilter::new(LevelFilter::Warn)))
            .build("logfile", Box::new(_logfile))
        )
        .build(Root::builder().appender("stdout").appender("logfile").build(LevelFilter::Info))
        .unwrap();

    _ = log4rs::init_config(config).unwrap();

    let listen_port = 1935;
    let address = format!("0.0.0.0:{port}", port = listen_port);
    let mut channel = ChannelsManager::new();
    let producer = channel.get_session_event_producer();
    let mut rtmp_server = RtmpServer::new(address, producer.clone());
    let mut cliet_event_receiver = channel.get_client_event_consumer();

    channel.set_rtmp_push_enabled(true);

    let channel_mx = Arc::new(Mutex::new(channel));
    let channel_mx2 = Arc::clone(&channel_mx);

    tokio::spawn(async move {
        loop  {
            match cliet_event_receiver.recv().await {
                Ok(message) => {
                    match message {
                        ClientEvent::Publish { app_name, stream_name } => {
                            log::debug!("!!!!!!!!!!!!!!!!! client event Publish - app: {}, stream: {}\n", app_name, stream_name);
                            match RtmpClient::new(
                                format!("localhost:{port}", port = listen_port),
                                app_name,
                                stream_name,
                            ).run().await {
                                Ok(()) => log::info!("RTMP Client completed"),
                                Err(error) => log::error!("subscribtion error: {:?}\n", error),
                            }

                            // let mut lock = channel_mx.lock().await;
                            // let subscription = lock.subscribe(&app_name, &stream_name, SessionInfo{
                            //     subscriber_id,
                            //     session_sub_type: SessionSubType::Player,
                            // }).await;
                            
                            // match subscription {
                            //     Ok(mut video_stream) => {
                            //         loop {
                            //             if let Some(val) = video_stream.recv().await {
                            //                 match val {
                            //                     ChannelData::MetaData { timestamp: _, data: _ } => {
                            //                         log::warn!("AAA AAA ChannelData::MetaData");
                            //                     }
                            //                     ChannelData::Audio { timestamp: _, data: _ } => {
                            //                         log::warn!("AAA AAA ChannelData::Audio");
                            //                     }
                            //                     ChannelData::Video { timestamp: _, data: _ } => {
                            //                         log::warn!("AAA AAA ChannelData::Video");
                            //                     }
                            //                 }
                            //            }
                            //         }
                            //     },
                            //     Err(error) => log::error!("!!!! !!!! subscribtion error: {:?}\n", error), 
                            // }
                        },
                        ClientEvent::UnPublish { app_name, stream_name } => {
                            log::debug!("!!!!!!!!!!!!!!!!! client event UnPublish - app: {}, stream: {}\n", app_name, stream_name);        
                        },
                        ClientEvent::Subscribe { app_name, stream_name } => {
                            log::debug!("!!!!!!!!!!!!!!!!! client event Subscribe - app: {}, stream: {}\n", app_name, stream_name);        
                        },
                        ClientEvent::UnSubscribe  { app_name, stream_name } => {
                            log::debug!("!!!!!!!!!!!!!!!!! client event UnSubscribe - app: {}, stream: {}\n", app_name, stream_name);        
                        }
                    }
                },
                Err(error) => log::error!("client event error: {:?}\n", error), 
            }
        };
    });

    tokio::spawn(async move {
        if let Err(err) = rtmp_server.run().await {
            log::error!("rtmp server error: {}\n", err);
        }
    });

    tokio::spawn(async move {
        let mut lock = channel_mx2.lock().await; 
        lock.run().await;
    });

    tokio::signal::ctrl_c().await?;

    Ok(())
}