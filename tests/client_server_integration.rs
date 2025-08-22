// tests/client_server_integration.rs
use netbeat::{BindInterface, Client, Server};
use std::thread;
use std::time::Duration;

#[test]
fn test_basic_client_server_flow() {
    // Start server in background
    let server = Server::builder()
        .interface(BindInterface::Localhost)
        .quiet(true)
        .build()
        .unwrap();

    let ip_addr = server.socket_addr.ip().to_string();
    let _server_handle = thread::spawn(move || {
        let _ = server.listen();
    });

    // Give server time to start
    thread::sleep(Duration::from_millis(100));

    // Connect client
    let client = Client::builder(ip_addr).quiet(true).build().unwrap();

    let result = client.contact();
    assert!(result.is_ok());

    // Validate results
    let report = result.unwrap();
    assert!(report.ping_report.successful_pings > 0);
    assert!(report.upload_report.bytes > 0);
    assert!(report.download_report.bytes > 0);
}

#[test]
fn test_multiple_clients() {
    let server = Server::builder()
        .interface(BindInterface::Localhost)
        .quiet(true)
        .build()
        .unwrap();

    let ip_addr = server.socket_addr.ip().to_string();
    let _server_handle = thread::spawn(move || {
        let _ = server.listen();
    });

    thread::sleep(Duration::from_millis(100));

    // Launch multiple clients
    let mut handles = vec![];

    for _ in 0..2 {
        let target_ip = ip_addr.clone();
        let handle = thread::spawn(move || {
            let client = Client::builder(target_ip).quiet(true).build().unwrap();

            client.contact()
        });
        handles.push(handle);

        // Stagger connections slightly
        thread::sleep(Duration::from_millis(50));
    }

    // Wait for all clients to complete
    for handle in handles {
        let result = handle.join().unwrap();
        assert!(result.is_ok());
    }
}
