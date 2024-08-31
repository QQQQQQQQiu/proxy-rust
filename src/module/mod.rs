mod cmd;
mod xhr;
mod doc;

pub use cmd::handle_command;
pub use cmd::handle_command_is_match_route;
pub use xhr::handle_xhr;
pub use xhr::handle_xhr_is_match_route;
pub use xhr::handle_xhr_is_pass_secret;
pub use doc::handle_doc;
pub use doc::handle_doc_is_match_route;