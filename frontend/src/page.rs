pub mod login;
pub mod register;
pub mod home;

pub use home::Home;
pub use login::Login;
pub use register::Register;


pub use route::*;

pub mod route {
    pub const ACCOUNT_LOGIN: &str = "/account/login";
    pub const ACCOUNT_REGISTER: &str = "/account/register";
    pub const HOME: &str = "/home";
}
