use futures_util::StreamExt;
use ox_core::agent::StreamEvent;
use ox_core::types::Message;
use ox_providers::{create_provider, LlmProvider, ProviderConfig, ProviderType};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

#[tokio::test]
async fn test_gemini_1_byte_chunk_stream() {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    // Spawn a mock HTTP server that writes a response 1 byte at a time
    tokio::spawn(async move {
        if let Ok((mut socket, _)) = listener.accept().await {
            // Read request
            let mut buf = [0; 1024];
            let _ = socket.read(&mut buf).await;

            let body = r#"data: {"candidates": [{"content": {"parts": [{"text": "Hello, "}],"role": "model"}}]}

data: {"candidates": [{"content": {"parts": [{"text": "world!"}],"role": "model"}}], "usageMetadata": {"promptTokenCount": 10, "candidatesTokenCount": 5}}

"#;
            let response = format!(
                "HTTP/1.1 200 OK\r\n\
                Content-Type: text/event-stream\r\n\
                Connection: close\r\n\
                \r\n\
                {}",
                body
            );

            // Write response 1 byte at a time to force chunk boundaries everywhere
            for byte in response.bytes() {
                let _ = socket.write_all(&[byte]).await;
                // yield to allow client to read chunks
                tokio::task::yield_now().await;
            }
        }
    });

    let config = ProviderConfig::new(ProviderType::Gemini, "gemini-3.6-flash")
        .with_api_key("dummy_key")
        .with_base_url(format!("http://{}", addr));

    let provider = create_provider(config).unwrap();
    let messages = vec![Message::user("Hi")];
    
    let mut stream = provider.stream_chat(&messages, &[]).await.expect("stream_chat failed");

    let mut full_text = String::new();
    let mut final_usage = None;

    while let Some(Ok(event)) = stream.next().await {
        match event {
            StreamEvent::TextDelta { text } => {
                full_text.push_str(&text);
            }
            StreamEvent::TurnCompleted { usage, .. } => {
                final_usage = Some(usage);
            }
            _ => {}
        }
    }

    assert_eq!(full_text, "Hello, world!");
    
    let usage = final_usage.expect("TurnCompleted was not emitted");
    assert_eq!(usage.input_tokens, 10);
    assert_eq!(usage.output_tokens, 5);
}
