use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct RunArgs {
    #[arg(short, long)]
    pub adress: String,
    #[arg(short, long, default_value_t = 8080)]
    pub port: u16,
    #[arg(long)]
    pub db: String,
    #[arg(long)]
    pub db_name: String,
}
