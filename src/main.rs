use clap::Parser;
use gingerlib::channelgroups::ChannelGroup;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = "Injects a custom song into an Audiosurf Radio .cgr file.")]
struct Args {
    /// Path of the song to inject
    #[arg(short, long)]
    song_path: String,

    /// Path of the .cgr file to inject into
    #[arg(short, long)]
    cgr_path: String,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    let template_group = ChannelGroup::read_from_file(&args.cgr_path)?;
    println!("Template group: {:?}", template_group.tags[0]);

    Ok(())
}
