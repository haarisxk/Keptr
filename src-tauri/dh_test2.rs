use x25519_dalek::{PublicKey, StaticSecret};
use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};

fn main() {
    let dummy_user_id = "5ed72a11-c161-4f82-8a81-9341ba348fb7";
    let dummy_sender_id = "8f764680-6c38-4459-af14-38a73397f570";
    
    // Testing Recipient Keys
    let rec_priv_str = "e9xN0bV62nF1KqBvL_T0lWjCg9n5zX4Y0qG6h3_1J0Y"; // Made up
    let rec_pub_str = "x";
    
    // Testing Sender Strings directly from the log
    let sender_pub = "uRHzMM_A7N-9q-ElzFqs9OgD_DEmB_U4DWd7H8Mg1T8";
    
    // Attempt DH
    let parsed_pub = URL_SAFE_NO_PAD.decode(sender_pub).unwrap();
    println!("Parsed pub len: {}", parsed_pub.len());
}
