use std::net::IpAddr;
use clap::Parser;
use lazy_static::lazy_static;

lazy_static! {
    pub static ref ARGS: Args = Args::parse();
}

#[derive(Parser, Debug, Clone)]
#[clap(author, version, about, long_about = None)]
pub struct Args {
    #[clap(long, env="MICROBIN_AUTH_USERNAME")]
    pub auth_username: Option<String>,

    #[clap(long, env="MICROBIN_AUTH_PASSWORD")]
    pub auth_password: Option<String>,

    #[clap(long, env="MICROBIN_EDITABLE")]
    pub editable: bool,

    #[clap(long, env="MICROBIN_FOOTER_TEXT")]
    pub footer_text: Option<String>,

    #[clap(long, env="MICROBIN_HIDE_FOOTER")]
    pub hide_footer: bool,

    #[clap(long, env="MICROBIN_HIDE_HEADER")]
    pub hide_header: bool,

    #[clap(long, env="MICROBIN_HIDE_LOGO")]
    pub hide_logo: bool,

    #[clap(long, env="MICROBIN_NO_LISTING")]
    pub no_listing: bool,

    #[clap(long, env="MICROBIN_HIGHLIGHTINGSYNTAX")]
    pub highlightsyntax: bool,

    #[clap(short, long, env="MICROBIN_PORT", default_value_t = 8080)]
    pub port: u16,

    #[clap(short, long, env="MICROBIN_BIND", default_value_t = IpAddr::from([0, 0, 0, 0]))]
    pub bind: IpAddr,

    #[clap(long, env="MICROBIN_PRIVATE")]
    pub private: bool,

    #[clap(long, env="MICROBIN_PURE_HTML")]
    pub pure_html: bool,

    #[clap(long, env="MICROBIN_READONLY")]
    pub readonly: bool,

    #[clap(long, env="MICROBIN_TITLE")]
    pub title: Option<String>,

    #[clap(short, long, env="MICROBIN_THREADS", default_value_t = 1)]
    pub threads: u8,

    #[clap(long, env="MICROBIN_WIDE")]
    pub wide: bool,
}