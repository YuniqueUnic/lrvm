pub enum LrvmMessage {
    Hello {
        alias: String,
    },
    HelloAck {
        /// current node alias
        alias: String,
        /// The others nodes (alias, IP, port)
        nodes: Vec<(String, String, String)>,
    },
}
