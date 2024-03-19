use filetime::FileTime;
use std::fs;
use std::path::Path;
use pk2::Pk2;
use pk2::fs::{DirEntry, Directory};
use clap::{crate_authors, crate_description, crate_name, crate_version};
use clap::{App, Arg, ArgMatches, SubCommand};

fn main() {
    let app = App::new(crate_name!())
        .version(crate_version!())
        .author(crate_authors!())
        .about(crate_description!())
        .subcommand(reroute_app());
    let matches = app.get_matches();
    match matches.subcommand() {
        ("reroute", Some(matches)) => reroute(matches),
        _ => println!("{}", matches.usage()),
    }
}

fn key_arg() -> Arg<'static, 'static> {
    Arg::with_name("blowfish_key")
        .short("k")
        .long("key")
        .takes_value(true)
        .env("PK2_BLOWFISH_KEY")
        .default_value("169841")
}

fn reroute_app() -> App<'static, 'static> {
    SubCommand::with_name("reroute")
        .version(crate_version!())
        .author(crate_authors!())
        .about(crate_description!())
        .arg(
            Arg::with_name("game_directory")
                .short("d")
                .long("game_directory")
                .required(true)
                .takes_value(true)
                .help("Sets the path to the game directory which should be rerouted"),
        )
        .arg(key_arg().help("Sets the blowfish key"))
}

fn reroute(matches: &ArgMatches<'static>) {
    let blowfish_key = matches.value_of("blowfish_key").unwrap().as_bytes();
    let game_directory_path = matches.value_of_os("game_directory").map(Path::new).unwrap();

    let temporary_extraction_path = game_directory_path.join("tmp_reroute_dir");
    if temporary_extraction_path.exists() {
        fs::remove_dir_all(&temporary_extraction_path).unwrap_or_else(|_| panic!("failed to remove temporary extraction directory at {:?}", temporary_extraction_path));
    }
    fs::create_dir(&temporary_extraction_path).unwrap_or_else(|_| panic!("failed to create temporary extraction directory at {:?}", temporary_extraction_path));

    // create backup of Media.pk2
    fs::copy(game_directory_path.join("Media.pk2"), game_directory_path.join("Media.pk2.bak")).unwrap_or_else(|_| panic!("failed to create backup of Media.pk2"));
    
    // extract Media.pk2
    let media_pk2_path = &game_directory_path.join("Media.pk2");

    let extracted_media_pk2_path = &temporary_extraction_path.join("Media");
    let media_pk2 = Pk2::open(media_pk2_path, blowfish_key).unwrap_or_else(|_| panic!("failed to open archive at {:?}", media_pk2_path));
    let media_pk2_root_directory = media_pk2.open_directory("/").unwrap();
    extract_files(media_pk2_root_directory, &extracted_media_pk2_path, false);

    // replace DIVISIONINFO.TXT
    let patched_divisioninfo = r#"   DIV01 	   127.0.0.1 "#;
    let divisioninfo_path = extracted_media_pk2_path.join("DIVISIONINFO.TXT");
    std::fs::write(&divisioninfo_path, patched_divisioninfo).unwrap();
    
    // pack Media.pk2
    let patched_media_pk2_path = &temporary_extraction_path.join("Media.pk2");
    let mut patched_media_pk2 = Pk2::create_new(patched_media_pk2_path, blowfish_key).unwrap_or_else(|_| panic!("failed to create archive at {:?}", patched_media_pk2_path));

    pack_files(&mut patched_media_pk2, &temporary_extraction_path.join("Media"), &temporary_extraction_path.join("Media"));

    // replace Media.pk2
    fs::remove_file(media_pk2_path).unwrap_or_else(|_| panic!("failed to remove archive at {:?}", media_pk2_path));
    fs::copy(patched_media_pk2_path, media_pk2_path).unwrap_or_else(|_| panic!("failed to copy archive to {:?}", media_pk2_path));

    let _ = fs::remove_dir_all(temporary_extraction_path);
}

fn pack_files(out_archive: &mut Pk2, dir_path: &Path, base: &Path) {
    use std::io::{Read, Write};
    let mut buf = Vec::new();
    for entry in std::fs::read_dir(dir_path).unwrap() {
        let entry = entry.unwrap();
        let ty = entry.file_type().unwrap();
        let path = entry.path();
        if ty.is_dir() {
            pack_files(out_archive, &path, base);
        } else if ty.is_file() {
            let mut file = std::fs::File::open(&path).unwrap();
            file.read_to_end(&mut buf).unwrap();
            out_archive
                .create_file(Path::new("/").join(path.strip_prefix(base).unwrap()))
                .unwrap()
                .write_all(&buf)
                .unwrap();
            buf.clear();
        }
    }
}

fn extract_files(folder: Directory<'_>, out_path: &Path, write_times: bool) {
    use std::io::Read;
    let _ = std::fs::create_dir(out_path);
    let mut buf = Vec::new();
    for entry in folder.entries() {
        match entry {
            DirEntry::File(mut file) => {
                file.read_to_end(&mut buf).unwrap();
                let file_path = out_path.join(file.name());
                if let Err(e) = std::fs::write(&file_path, &buf) {
                    eprintln!("Failed writing file at {:?}: {}", file_path, e);
                } else if write_times {
                    if let Some(time) = file.modify_time() {
                        let _ =
                            filetime::set_file_mtime(&file_path, FileTime::from_system_time(time));
                    }
                    if let Some(time) = file.access_time() {
                        let _ =
                            filetime::set_file_atime(&file_path, FileTime::from_system_time(time));
                    }
                }
                buf.clear();
            }
            DirEntry::Directory(dir) => {
                let dir_name = dir.name();
                let path = out_path.join(dir_name);
                extract_files(dir, &path, write_times);
            }
        }
    }
}
