use std::{ffi::OsStr, fs, path::PathBuf};

use anyhow::anyhow;
use clap::Parser;
use gingerlib::channelgroups::ChannelGroup;

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
    song_path: PathBuf,

    /// Path of the Audiosurf highway to inject
    #[arg(short, long)]
    ash_path: Option<String>,

    /// Path of the .cgr file to inject into
    #[arg(short, long)]
    cgr_path: Option<String>,

    /// Path of the output .cgr file
    #[arg(short, long)]
    output_path: String,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let song_data = fs::read(&args.song_path)?;
    let ash_data = match &args.ash_path {
        Some(path) => fs::read(path)?,
        None => {
            println!("No .ash file specified, trying to find it in game directory");
            let steam_dir = steamlocate::SteamDir::locate()?;
            let (audiosurf, library) = steam_dir.find_app(12900)?.unwrap();
            let mut audiosurf_path = library.resolve_app_dir(&audiosurf);
            audiosurf_path.extend(["engine", "AudiosurfHC"]);
            let mut ash_data: Vec<u8> = Vec::new();
            // read directory
            for entry in fs::read_dir(audiosurf_path)? {
                let entry = entry?;
                let path = entry.path();
                // check if file is .ash
                // and if the file name contains the song name
                if path.extension() == Some(OsStr::new("ash"))
                    && path
                        .file_stem()
                        .unwrap()
                        .to_str()
                        .unwrap()
                        .contains(args.song_path.file_name().unwrap().to_str().unwrap())
                {
                    println!("Found .ash candidate at {}", path.to_str().unwrap());
                    ash_data = fs::read(path)?;
                    break;
                }
            }
            if ash_data.is_empty() {
                return Err(anyhow!("Couldn't find .ash file!"));
            }
            ash_data
        }
    };

    let mut template_group = match &args.cgr_path {
        Some(path) => ChannelGroup::read_from_file(path)?,
        None => {
            println!("No .cgr file specified, attempting to locate suitable file from Audiosurf");
            let steam_dir = steamlocate::SteamDir::locate()?;
            let (audiosurf, library) = steam_dir.find_app(12900)?.unwrap();
            let mut audiosurf_path = library.resolve_app_dir(&audiosurf);
            audiosurf_path.extend([
                "engine",
                "Cache",
                "Web",
                "www.audio-surf.com",
                "as",
                "asradio",
                "ASR_PedroCamacho_AudiosurfOverture.cgr",
            ]);
            let audiosurf_path_str = audiosurf_path.to_str().unwrap();
            //surely this is always UTF-8 :^)
            println!("Found file at {}", audiosurf_path_str);
            ChannelGroup::read_from_file(audiosurf_path_str)?
        }
    };

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
