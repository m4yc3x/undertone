use std::sync::Arc;
use futures::{SinkExt, StreamExt};
use serde_json::json;
use webrtc::{
    api::APIBuilder,
    data_channel::{
        data_channel_init::RTCDataChannelInit,
        data_channel_message::DataChannelMessage,
    },
    ice_transport::ice_candidate::RTCIceCandidate,
    peer_connection::{
        offer_answer_options::RTCOfferOptions, configuration::RTCConfiguration,
    },
};
use rand::Rng;
use rand::distributions::Alphanumeric;
use tokio::sync::Mutex as AsyncMutex;
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message, WebSocketStream, MaybeTlsStream};
use crate::utility::{log, err};
use crate::networking::bootstrap::Message as UTPacket;

pub async fn initialize(connect_to: String, secure_mode: bool) -> Result<(), Box<dyn std::error::Error>> {
    let signaling_server_url = "ws://127.0.0.1:1720";
    
    let (ws_stream, _) = connect_async(signaling_server_url).await?;
    let ws_stream = Arc::new(AsyncMutex::new(ws_stream));

    let mut id_length = 12;
    if secure_mode {
        id_length = 48;
    }

    // Generate a random client ID
    let client_id: String = rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(id_length)
        .map(char::from)
        .collect();

    log(&format!("Your ID is: {}", client_id.clone()));

    // Register with the signaling server
    let register_message = UTPacket {
        msg_type: Some("register".to_string()),
        id: Some(client_id.clone()),
        to: None,
        from: None,
        payload: None,
    };
    
    let register_message_str = serde_json::to_string(&register_message).unwrap();
    ws_stream.lock().await.send(Message::Text(register_message_str)).await?;
    
    // Wait to handle an offer
    while let Some(message) = ws_stream.lock().await.next().await {
        match message {
            Ok(msg) => {
                if msg.is_text() {
                    let msg_text = msg.to_text().unwrap();
                    let parsed_msg: serde_json::Value = serde_json::from_str(msg_text).unwrap();
                    match parsed_msg["type"].as_str() {
                        Some("offer") => {
                            let _offer = parsed_msg["offer"].as_str().unwrap();
                            let _from = parsed_msg["from"].as_str().unwrap();
                            // handle offer
                        },
                        Some("hello") => {
                            log("Connected to signaling server! Awaiting connection...");
                            if connect_to != "disabled" {
                                log("Attempting to connect to peer...");
                                create_offer(ws_stream.clone(), client_id.clone(), connect_to.clone()).await?;
                                log("yes");
                            }
                        },
                        _ => {}
                    }
                }
            },
            Err(e) => {
                err(&format!("Error receiving message: {}", e));
                break;
            }
        }
    }
        
    Ok(())
}

async fn create_offer(
    ws_stream: Arc<AsyncMutex<WebSocketStream<MaybeTlsStream<tokio::net::TcpStream>>>>,
    client_id: String,
    to: String,
) -> Result<(), Box<dyn std::error::Error>> {
    // Create a new RTCPeerConnection
    let api = APIBuilder::new().build();
    let config = RTCConfiguration::default();
    let peer_connection = api.new_peer_connection(config).await?;

    
    // Create a new data channel
    let data_channel_init = RTCDataChannelInit {
        ordered: Some(true),
        max_packet_life_time: None,
        max_retransmits: None,
        protocol: Some("".to_string()),
        negotiated: None,
    };
    let data_channel = peer_connection
        .create_data_channel("data", Some(data_channel_init))
        .await?;

    // Create an offer
    let offer_options = RTCOfferOptions::default();
    let offer = peer_connection.create_offer(Some(offer_options)).await?;
    peer_connection.set_local_description(offer.clone()).await?;
        

    // Send the offer to the signaling server
    let offer_message = UTPacket {
        msg_type: Some("offer".to_string()),
        id: Some(client_id.clone()),
        to: Some(to.clone()),
        from: Some(client_id.clone()),
        payload: Some(json!({ "offer": offer.sdp }).to_string()),
    };
    let offer_message_str = serde_json::to_string(&offer_message).unwrap();
    ws_stream.lock().await.send(Message::Text(offer_message_str)).await?;
    println!("hello");

    // Handle ICE candidates
    peer_connection
        .on_ice_candidate(Box::new(move |ice_candidate: Option<RTCIceCandidate>| {
            let ws_stream = ws_stream.clone();
            let client_id = client_id.clone();
            let to = to.clone();
            Box::pin(async move {
                if let Some(ice_candidate) = ice_candidate {
                    let ice_candidate_message = UTPacket {
                        msg_type: Some("ice_candidate".to_string()),
                        id: Some(client_id),
                        to: Some(to),
                        from: None,
                        payload: Some(json!({ "candidate": ice_candidate.stats_id }).to_string()),
                    };
                    let ice_candidate_message_str = serde_json::to_string(&ice_candidate_message).unwrap();
                    ws_stream.lock().await.send(Message::Text(ice_candidate_message_str)).await.unwrap();
                }
            })
        }));

    // Handle data channel events
    data_channel.on_open(Box::new(move || {
        Box::pin(async move {
            log("Data channel opened!");
        })
    }));

    data_channel.on_close(Box::new(move || {
        Box::pin(async move {
            log("Data channel closed!");
        })
    }));

    data_channel.on_message(Box::new(move |message: DataChannelMessage| {
        Box::pin(async move {
            log(&format!("Received message: {:?}", message));
        })
    }));

    Ok(())
}