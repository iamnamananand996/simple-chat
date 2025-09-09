// Main test file - serves as an entry point and overview of the test suite
//
// Test Organization:
// - integration_tests.rs: Basic server functionality and integration tests
// - stress_tests.rs: High throughput, load testing, and performance tests
// - tracing_tests.rs: Detailed logging and message flow verification tests
//
// Run all tests: cargo test
// Run specific test file: cargo test --test integration_tests
// Run with output: cargo test -- --nocapture

use chat_server::websocket::run_chat_server;

#[tokio::test]
async fn test_functional_approach() {
    // Test that we can create user stores and handle basic operations
    use chat_server::users::{create_user_stores, add_user, get_user};
    
    let (users, usernames) = create_user_stores();
    
    // Test adding a user
    let user_id = add_user(&users, &usernames, "test_user".to_string()).unwrap();
    
    // Test getting the user
    let user = get_user(&users, user_id).unwrap();
    assert_eq!(user.username, "test_user");
}

#[tokio::test] 
async fn test_duplicate_username() {
    // Test that duplicate usernames are rejected
    use chat_server::users::{create_user_stores, add_user};
    
    let (users, usernames) = create_user_stores();
    
    // Add first user
    add_user(&users, &usernames, "duplicate".to_string()).unwrap();
    
    // Try to add second user with same name - should fail
    let result = add_user(&users, &usernames, "duplicate".to_string());
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("already taken"));
}
