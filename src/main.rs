use std::fs;

use anyhow::anyhow;
use clap::Parser;
use gingerlib::channelgroups::ChannelGroup;
//use lofty::{read_from_path, TaggedFileExt};

#[derive(Parser, Debug)]
#[command(
    author,
    version,
    about,
    long_about = "Inject a custom song into an Audiosurf Radio .cgr file!"
)]
struct Args {
    /// Path of the song to inject
    #[arg(short, long)]
    song_path: String,

    /// Path of the Audiosurf highway to inject
    #[arg(short, long)]
    ash_path: String,

    /// Path of the .cgr file to inject into
    #[arg(short, long)]
    cgr_path: String,

    /// Path of the output .cgr file
    #[arg(short, long)]
    output_path: String,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let song_data = fs::read(&args.song_path)?;
    let ash_data = fs::read(&args.ash_path)?;

    let mut template_group = ChannelGroup::read_from_file(&args.cgr_path)?;
    if template_group.tags[33].name != "BUFS" {
        return Err(anyhow!("Couldn't find song buffer vault size tag!"));
    }
    println!(
        "Current song buffer vault size: {}",
        u32::from_le_bytes(template_group.tags[33].data.as_slice().try_into()?)
    );
    template_group.tags[33].data = song_data.len().to_le_bytes().to_vec();
    println!("Changing song buffer vault size to {}", song_data.len());

    if template_group.tags[34].name != "BUFV" {
        return Err(anyhow!("Couldn't find song buffer to inject into!"));
    }
    println!("Injecting song data");
    template_group.tags[34].data = song_data;

    if template_group.tags[161].name != "BUFS" {
        return Err(anyhow!("Couldn't find highway buffer vault size tag!"));
    }
    println!(
        "Current highway buffer vault size: {}",
        u32::from_le_bytes(template_group.tags[161].data.as_slice().try_into()?)
    );
    template_group.tags[161].data = ash_data.len().to_le_bytes().to_vec();
    println!("Changing highway buffer vault size to {}", ash_data.len());

    if template_group.tags[162].name != "BUFV" {
        return Err(anyhow!("Couldn't find highway buffer to inject into!"));
    }
    println!("Injecting highway data");
    template_group.tags[162].data = ash_data;

    println!("Done! Writing to {}", args.output_path);
    template_group.save_to_file(&args.output_path)?;

    Ok(())
}
