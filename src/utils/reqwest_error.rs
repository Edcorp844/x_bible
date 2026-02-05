pub trait ReqwestErrorExt {
    fn to_user_friendly_message(&self) -> String;
}

impl ReqwestErrorExt for reqwest::Error {
    fn to_user_friendly_message(&self) -> String {
        if self.is_connect() || self.is_request() {
             "Connection Error. Please check your internet connection and try again".to_string()
        } else if self.is_timeout() {
            "The request timed out. The news server might be slow right now.".to_string()
        } else if self.is_decode() {
            "Received an unexpected response format from the server.".to_string()
        } else {
            "A network error occurred. Please try again later.".to_string()
        }
    }
}