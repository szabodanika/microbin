use clap::Parser;
use lazy_static::lazy_static;

lazy_static! {
    pub static ref ARGS: Args = Args::parse();
}

#[derive(Parser, Debug, Clone)]
#[clap(author, version, about, long_about = None)]
pub struct Args {
    #[clap(long)]
    pub auth_username: Option<String>,

    #[clap(long)]
    pub auth_password: Option<String>,

    #[clap(long)]
    pub editable: bool,

    #[clap(long)]
    pub footer_text: Option<String>,

    #[clap(long)]
    pub hide_footer: bool,

    #[clap(long)]
    pub hide_header: bool,

    #[clap(long)]
    pub hide_logo: bool,

    #[clap(long)]
    pub no_listing: bool,

    #[clap(long)]
    pub highlightsyntax: bool,

    #[clap(short, long, default_value_t = 8080)]
    pub port: u32,

    #[clap(long)]
    pub private: bool,

    #[clap(long)]
    pub pure_html: bool,

    #[clap(long)]
    pub readonly: bool,

    #[clap(long)]
    pub title: Option<String>,

    #[clap(short, long, default_value_t = 1)]
    pub threads: u8,

    #[clap(long)]
    pub wide: bool,
}
