use {
    super::session::RtmpSession,
    rtmp::relay::errors::ClientError,
    tokio::net::TcpStream,
};

pub struct RtmpClient {
    address: String,
    app_name: String,                        
    stream_name: String,
}

impl RtmpClient {
    pub fn new(
        address: String,
        app_name: String,                        
        stream_name: String,
    ) -> Self {
        Self {
            address,
            app_name,
            stream_name,
        }
    }

    pub async fn run(&mut self) -> Result<(), ClientError> {
        let stream = TcpStream::connect(self.address.clone()).await?;

        let mut client_session = RtmpSession::new(
            stream,
            self.app_name.clone(),
            self.stream_name.clone(),
        );

        if let Err(err) = client_session.run().await {
            log::error!("rtmp client run error: {}", err);
        }

        Ok(())
    }
}
