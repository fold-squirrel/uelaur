use edit_distance::edit_distance;
use serde::Deserialize;
use std::ffi::OsStr;
use std::path::PathBuf;
use std::sync::{mpsc, Mutex};
use std::time::Duration;
use std::{f32, fs};
use std::{
    io::{self, Write},
    process::Command,
};
use std::{sync, thread, time};

const SHORT_DUR: std::time::Duration = {
    if cfg!(target_os = "windows") {
        Duration::new(10, 0)
    } else if cfg!(target_os = "linux") {
        Duration::new(2, 0)
    } else if cfg!(target_os = "macos") {
        Duration::new(5, 0)
    } else {
        Duration::MAX
    }
};

const MEDIUM_DUR: std::time::Duration = {
    if cfg!(target_os = "windows") {
        Duration::new(40, 0)
    } else if cfg!(target_os = "linux") {
        Duration::new(10, 0)
    } else if cfg!(target_os = "macos") {
        Duration::new(20, 0)
    } else {
        Duration::MAX
    }
};

const LONG_DUR: std::time::Duration = {
    if cfg!(target_os = "windows") {
        Duration::new(120, 0)
    } else if cfg!(target_os = "linux") {
        Duration::new(15, 0)
    } else if cfg!(target_os = "macos") {
        Duration::new(30, 0)
    } else {
        Duration::MAX
    }
};

fn main() {
    let default_panic = std::panic::take_hook();
    if cfg!(target_os = "windows") {
        std::panic::set_hook(Box::new(|_| {
            println!(
                "read or write operation failed\n\n\
                      maybe an anti-virus is preving uelaur from accessing files on this computer \
                      or the windows blocked access to current folder\n\n\
                      temporarily disable your anti-virus or if you ran the program \
                      inside a protected folder like 'Documents', 'Pictures', 'Videos', 'Music', \
                      'Favorites', 'Downloads' then consider creating a new folder \
                      that is not inside any of the previously listed folders"
            );
            thread::sleep(LONG_DUR);
        }));
    };
    if cfg!(target_os = "linux") {
        std::panic::set_hook(Box::new(|_| {
            println!("read/write operation failed: permission denied");
            thread::sleep(LONG_DUR);
        }));
    };
    if cfg!(target_os = "macos") {
        std::panic::set_hook(Box::new(|_| {
            println!(
                "read or write operation failed\n\n\
                      maybe an anti-virus is preving uelaur from accessing files on this mac \
                      or the the current user doesn't have read and write permission in \
                      the current directory\n\n\
                      temporarily disable your anti-virus or if you ran the program \
                      inside a directory that you don't have read and write permission in then \
                      make a new directory and run uelaur there"
            );
            thread::sleep(LONG_DUR);
        }));
    };

    let entries = ls(".");
    let csv_files: Vec<std::path::PathBuf>;
    let mut config: Config;

    if entries.len() == 1 {
        expand1().unwrap();
        println!("new project intialized");
        thread::sleep(SHORT_DUR);
        return;
    }

    if entries.contains(&"./uelaur_config.txt".into()) {
        let fs = fs::read("uelaur_config.txt").unwrap();
        let file = std::str::from_utf8(&fs).unwrap();
        config = match toml::from_str(file) {
            Ok(c) => c,
            Err(_) => {
                if entries.contains(&"./config_backup.txt".into()) {
                    fs::write("./uelaur_config.txt", gen_helper_config(file.to_owned())).unwrap();
                    println!(
                        "uelaur_config.txt file could not be parsed, please fix \
                             all errors first, if you need help you send an email to \
                             Ahmed_Alaa_Gomaa@outlook.com"
                    );
                } else {
                    fs::write("./config_backup.txt", file).unwrap();
                    fs::write("./uelaur_config.txt", gen_helper_config(file.to_owned())).unwrap();
                    println!(
                        "uelaur_config.txt file could not be parsed,\n\
                             uelaur_config.txt was renamed to config_backup.txt\n\
                             and a new uelaur_config.txt was created \
                             containing more information about the errors"
                    );
                }
                thread::sleep(MEDIUM_DUR);
                return;
            }
        };

        if config.horizontal_marks.len() != config.horizontal_postions.len()
            || config.vertical_marks.len() != config.vertical_postions.len()
        {
            if config.horizontal_marks.len() != config.horizontal_postions.len() {
                println!(
                    "error: horizontal_postions and horizontal_marks must have \
                         the same length"
                );
            }
            if config.vertical_marks.len() != config.vertical_postions.len() {
                println!("error: vertical_postions and vertical_marks must have the same length");
            }
            thread::sleep(MEDIUM_DUR);
            return;
        }
    } else {
        println!(
            "uelaur must be run in an empty folder\nCreate a new folder \
                 then move uelaur there then run it to create a new project"
        );
        thread::sleep(MEDIUM_DUR);
        return;
    };

    if entries.contains(&"./csv_files".into()) {
        csv_files = ls("./csv_files");
        if csv_files.is_empty() {
            println!("place all the csv file in the csv_files folder");
            thread::sleep(MEDIUM_DUR);
            return;
        }
    } else {
        println!("place all the csv file in the csv_files folder");
        fs::create_dir("./csv_files").unwrap();
        thread::sleep(MEDIUM_DUR);
        return;
    }

    let (trans, recv) = sync::mpsc::channel::<Action>();
    let (finish, thread_done) = sync::mpsc::channel::<u8>();

    if get_pdf_path(entries.to_owned()).is_none() {
        println!("place the uel pdf in this folder");
        thread::sleep(MEDIUM_DUR);
        return;
    }

    let panic_trans = Mutex::new(trans.clone());
    let ui_done = Mutex::new(thread_done);
    if cfg!(target_os = "windows") {
        std::panic::set_hook(Box::new(move |info| {
            let panic_trans = panic_trans.lock().unwrap();
            panic_trans
                .send(Action::Msg(Message::Fail(
                    "an unexpected error occurred".to_owned(),
                )))
                .unwrap();
            panic_trans
                .send(Action::Msg(Message::Warn(
                    "this window will close in 2 minutes\n\
                                                      please copy the following debug message \
                                                      and send it to \
                                                      Ahmed_Alaa_Gomaa@outlook.com"
                        .to_owned(),
                )))
                .unwrap();
            panic_trans.send(Action::Panic).unwrap();
            ui_done.lock().unwrap().recv().unwrap();
            default_panic(info);
            thread::sleep(LONG_DUR);
            println!("exiting");
        }));
    } else if cfg!(target_os = "linux") {
        std::panic::set_hook(Box::new(move |info| {
            let panic_trans = panic_trans.lock().unwrap();
            panic_trans
                .send(Action::Msg(Message::Fail(
                    "an unexpected error occurred".to_owned(),
                )))
                .unwrap();
            panic_trans
                .send(Action::Msg(Message::Warn(
                    "the process will terminate in 15 seconds\n\
                                                      please copy the following debug message \
                                                      and send it to \
                                                      Ahmed_Alaa_Gomaa@outlook.com"
                        .to_owned(),
                )))
                .unwrap();
            panic_trans.send(Action::Panic).unwrap();
            ui_done.lock().unwrap().recv().unwrap();
            default_panic(info);
            thread::sleep(LONG_DUR);
            println!("exiting");
        }));
    } else if cfg!(target_os = "macos") {
        std::panic::set_hook(Box::new(move |info| {
            let panic_trans = panic_trans.lock().unwrap();
            panic_trans
                .send(Action::Msg(Message::Fail(
                    "an unexpected error occurred".to_owned(),
                )))
                .unwrap();
            panic_trans
                .send(Action::Msg(Message::Warn(
                    "the process will terminate in 30 seconds\n\
                                                      please copy the following debug message \
                                                      and send it to \
                                                      Ahmed_Alaa_Gomaa@outlook.com"
                        .to_owned(),
                )))
                .unwrap();
            panic_trans.send(Action::Panic).unwrap();
            ui_done.lock().unwrap().recv().unwrap();
            default_panic(info);
            thread::sleep(LONG_DUR);
            println!("exiting");
        }));
    };

    let handle = init_ui(recv, finish);

    if entries.contains(&"./uel_pdf.patched".into()) {
        expand2().unwrap();
        trans
            .send(Action::Msg(Message::Succ(
                "found uel_pdf.patched".to_owned(),
            )))
            .unwrap();
    } else if let Some(pdf_path) = get_pdf_path(entries) {
        expand2().unwrap();
        trans
            .send(Action::Update("patching pdf".to_owned()))
            .unwrap();
        patch_pdf(&pdf_path).unwrap();
        trans
            .send(Action::Msg(Message::Succ(
                "created uel_pdf.patched".to_owned(),
            )))
            .unwrap();
    }

    trans
        .send(Action::Update("updating config options".to_owned()))
        .unwrap();
    update_config(&mut config);
    trans
        .send(Action::Msg(Message::Succ(
            "all config options valid".to_owned(),
        )))
        .unwrap();

    trans
        .send(Action::Update("collecting csv file data".to_owned()))
        .unwrap();

    let mut csv_errors = vec![];
    let mut all_csv_data = vec![];
    for csv_file in csv_files {
        let file_name = csv_file.to_str().unwrap().to_owned();
        trans
            .send(Action::Update(format!("serializing {}", file_name)))
            .unwrap();
        let mut path = std::path::PathBuf::from("uel_papers");
        let mut dir = csv_file.clone();
        dir.set_extension("");
        path.push(dir.file_name().unwrap());
        if fs::read_dir(&path).is_err() {
            fs::create_dir(&path).unwrap();
        }

        match get_all_records(csv_file, &config) {
            Ok((data, err)) => {
                match err.len() {
                    0 => {
                        trans
                            .send(Action::Msg(Message::Succ(format!(
                                "serialization of {} complete, not errors to report",
                                file_name,
                            ))))
                            .unwrap();
                    }

                    1 => {
                        trans
                            .send(Action::Msg(Message::Warn(format!(
                                "serialization of {} complete, 1 error occured",
                                file_name,
                            ))))
                            .unwrap();
                        trans
                            .send(Action::Msg(Message::Warn("continuing anyway".to_owned())))
                            .unwrap();
                        csv_errors.push((file_name, err));
                    }

                    _ => {
                        trans
                            .send(Action::Msg(Message::Warn(format!(
                                "serialization of {} complete, {} errors occured",
                                file_name,
                                err.len()
                            ))))
                            .unwrap();
                        trans
                            .send(Action::Msg(Message::Warn("continuing anyway".to_owned())))
                            .unwrap();
                        csv_errors.push((file_name, err));
                    }
                }

                all_csv_data.push((path, data));
            }
            Err(err) => {
                trans
                    .send(Action::Msg(Message::Fatal(
                        "failed to parse full_mark, \
                                                 file can't be serialized"
                            .to_owned(),
                    )))
                    .unwrap();
                trans
                    .send(Action::Msg(Message::Warn(format!(
                        "skipping {}",
                        file_name
                    ))))
                    .unwrap();
                csv_errors.push((file_name, vec![err]));
            }
        };
    }

    if !csv_errors.is_empty() {
        let mut csv_errors_content = "".to_owned();

        for (csv_file_name, errors_vec) in csv_errors {
            let mut s = ' ';
            if errors_vec.len() > 1 {
                s = 's';
            }
            csv_errors_content.push_str(&format!(
                "\nIn: {}, found {} error{}\n",
                csv_file_name,
                errors_vec.len(),
                s
            ));
            for err in errors_vec {
                csv_errors_content.push_str(&format!("\n{}", err));
            }

            csv_errors_content.push('\n');
        }

        fs::write("./CSV_ERRORS.txt", csv_errors_content).unwrap();
    }

    if !all_csv_data.is_empty() {
        trans
            .send(Action::Msg(Message::Succ("collected csv data".to_owned())))
            .unwrap();
    } else {
        trans
            .send(Action::Msg(Message::Fatal(
                "All csv files invalid".to_owned(),
            )))
            .unwrap();
        trans
            .send(Action::Msg(Message::Warn(
                "nothing to do\nexiting soon".to_owned(),
            )))
            .unwrap();
        trans.send(Action::Panic).unwrap();
        handle.join().unwrap();
        thread::sleep(MEDIUM_DUR);
        return;
    }

    trans
        .send(Action::Update("creating uel papers".to_owned()))
        .unwrap();
    for (path, data) in all_csv_data {
        for arguments in gen_leafedit_operations(path, &config, data) {
            if cfg!(target_os = "windows") {
                Command::new("leafedit.exe")
                    .args(arguments)
                    .output()
                    .unwrap();
            } else {
                Command::new("./leafedit").args(arguments).output().unwrap();
            }
        }
    }
    trans
        .send(Action::Msg(Message::Succ(
            "all uel pdfs created".to_owned(),
        )))
        .unwrap();

    trans
        .send(Action::Update("merging pdfs".to_owned()))
        .unwrap();
    for pdf_dir in ls("./uel_papers") {
        let all_pdfs = ls(pdf_dir.to_str().unwrap());
        if !all_pdfs.is_empty() {
            let mut merged_path = PathBuf::from("./review");
            merged_path.push(pdf_dir.file_name().unwrap());
            merged_path.set_extension("pdf");
            if cfg!(target_os = "windows") {
                Command::new("leafedit.exe")
                    .arg("merge")
                    .args(all_pdfs)
                    .arg(merged_path)
                    .output()
                    .unwrap();
            } else {
                Command::new("./leafedit")
                    .arg("merge")
                    .args(all_pdfs)
                    .arg(merged_path)
                    .output()
                    .unwrap();
            }
        }
    }
    trans
        .send(Action::Msg(Message::Succ(
            "merged pdf and placed them in review \
                                        folder for easy reviewing\n\
                                        please make sure all the information in the pdfs \
                                        is correct before sending them"
                .to_owned(),
        )))
        .unwrap();

    trans.send(Action::Quit).unwrap();
    handle.join().unwrap();
    thread::sleep(SHORT_DUR);
}

type Data = (String, String, f32, f32);

fn gen_leafedit_operations(dir: PathBuf, config: &Config, data: Vec<Data>) -> Vec<Vec<String>> {
    let mut operation = vec![];
    for (name, id, mark, full_mark) in data {
        let mut formated_name = "".to_owned();
        for word in name.split_whitespace() {
            let mut chs = word.chars();
            formated_name.push(chs.next().unwrap().to_ascii_uppercase());
            formated_name.push_str(&chs.map(|f| f.to_ascii_lowercase()).collect::<String>());
            formated_name.push(' ');
        }
        formated_name.pop();
        let mut file_name = dir.clone();
        file_name.push(&format!("{}_{}.pdf", formated_name.replace(' ', "_"), id));

        let first_and_second_marker = if mark % 1_f32 < 0.1 {
            format!("{}/{}", mark as u64, full_mark as u64)
        } else {
            format!("{:.1}/{}", mark, full_mark as u64)
        };

        let asu_mark = ((mark / full_mark) * 100_f32).clamp(0_f32, 100_f32);
        let horizontal_index = get_horizontal_mark_index(&config.horizontal_marks, asu_mark);
        let vertical_index = get_vertical_mark_index(&config.vertical_marks, asu_mark);
        let uel_mark = get_uel_mark(&config.vertical_marks, asu_mark, vertical_index);
        let asu_mark = format!("{:.2}", asu_mark);
        let uel_mark = format!("{:.2}", uel_mark);

        operation.push(vec![
            "edit".to_string(),
            "-o".to_string(),
            format!(
                "Wr({},{},{},\"{}\")",
                config.name_postion[0],
                config.name_postion[1],
                config.student_field_font_size,
                formated_name
            ),
            "-o".to_string(),
            format!(
                "Wr({},{},{},\"{}\")",
                config.id_postion[0], config.id_postion[1], config.student_field_font_size, id
            ),
            "-o".to_string(),
            format!(
                "Wr({},{},{},\"\\u{{2713}}\")",
                config.horizontal_postions[horizontal_index][0],
                config.horizontal_postions[horizontal_index][1],
                config.horizontal_feild_font_size
            ),
            "-o".to_string(),
            format!(
                "Wr({},{},{},\"{}\")",
                config.first_marker_postion[0],
                config.first_marker_postion[1],
                config.grade_field_font_size,
                first_and_second_marker
            ),
            "-o".to_string(),
            format!(
                "Wr({},{},{},\"{}\")",
                config.second_marker_postion[0],
                config.second_marker_postion[1],
                config.grade_field_font_size,
                first_and_second_marker
            ),
            "-o".to_string(),
            format!(
                "Wr({},{},{},\"{}\")",
                config.asu_mark_postion[0],
                config.asu_mark_postion[1],
                config.grade_field_font_size,
                asu_mark
            ),
            "-o".to_string(),
            format!(
                "Wr({},{},{},\"{}\")",
                config.uel_mark_postion[0],
                config.uel_mark_postion[1],
                config.grade_field_font_size,
                uel_mark
            ),
            "-o".to_string(),
            format!(
                "Wr({},{},{},\"\\u{{2713}}\")",
                config.vertical_postions[vertical_index][0],
                config.vertical_postions[vertical_index][1],
                config.vertical_feild_font_size
            ),
            "uel_pdf.patched".to_string(),
            file_name.to_str().unwrap().to_string(),
        ])
    }
    operation
}

fn get_uel_mark(vertical_marks: &[[usize; 2]], mark: f32, index: usize) -> f32 {
    let (uel_less, asu_less) = if index == vertical_marks.len() - 1 {
        (0_f32, 0_f32)
    } else {
        (
            vertical_marks[index + 1][0] as f32,
            vertical_marks[index + 1][1] as f32,
        )
    };

    let (uel, asu) = (
        vertical_marks[index][0] as f32,
        vertical_marks[index][1] as f32,
    );

    ((mark - asu_less) / (asu - asu_less)) * (uel - uel_less) + uel_less
}

fn get_vertical_mark_index(vertical_marks: &[[usize; 2]], mark: f32) -> usize {
    let mut index = vertical_marks.len();
    for (i, seg) in vertical_marks.iter().enumerate() {
        if mark as usize > seg[1] {
            index = i;
            break;
        }
    }
    index - 1
}

fn get_horizontal_mark_index(horizontal_marks: &[usize], mark: f32) -> usize {
    let mut index = horizontal_marks.len() - 1;
    for (i, seg) in horizontal_marks.iter().enumerate() {
        if mark as usize >= *seg {
            index = i;
            break;
        }
    }
    index
}

fn get_all_records(csv: PathBuf, config: &Config) -> Result<(Vec<Data>, Vec<String>), String> {
    let mut errors = vec![];

    let mut csv_reader = csv::ReaderBuilder::new()
        .flexible(true)
        .from_path(&csv)
        .unwrap();

    let mut vec = vec![];
    let mut full_mark = None;
    for (i, record) in csv_reader.records().enumerate() {
        match record {
            Ok(record) => {
                if full_mark.is_none() {
                    match record.get(config.final_mark_column) {
                        Some(temp) => {
                            full_mark = match temp.parse::<f32>() {
                                Ok(n) => Some(n),
                                Err(_) => {
                                    if let Ok(n) = temp.parse::<u16>() {
                                        Some(n as f32)
                                    } else {
                                        return Err(format!(
                                            "FATAL: line {}, failed to parse final \
                                                    mark to a number (int/float)",
                                            i + 2
                                        ));
                                    }
                                }
                            }
                        }
                        None => {
                            return Err(format!(
                                "FATAL: line {}, final mark was not founld at column {}",
                                i + 2,
                                config.final_mark_column + 1
                            ))
                        }
                    }
                }

                let temp = record.get(config.mark_column).unwrap_or("0.0");
                let mark = temp
                    .parse::<f32>()
                    .unwrap_or_else(|_| temp.parse::<u16>().unwrap_or(0) as f32);

                let name = record.get(config.name_column).unwrap_or("").to_owned();
                let id = record.get(config.id_column).unwrap_or("").to_owned();

                if name.is_empty() && id.is_empty() {
                    errors.push(format!("line {}, no name found, no id found", i + 2));
                } else if name.is_empty() {
                    errors.push(format!("line {}, no name found", i + 2));
                } else if id.is_empty() {
                    errors.push(format!("line {}, no id found", i + 2));
                } else {
                    vec.push((name, id, mark, full_mark.unwrap()))
                }
            }
            Err(_) => {
                errors.push(format!("line: {}, could not be serialized", i + 2));
            }
        };
    }
    Ok((vec, errors))
}

fn update_config(config: &mut Config) {
    let page_height = pdf_height().unwrap();

    let update_csv = |num: &mut usize| {
        if *num > 0 {
            *num -= 1
        }
    };

    let update_coords = |num: &mut [usize; 2]| {
        let height = pdf_height as usize;
        if num[1] <= height {
            num[1] = height - num[1]
        }
    };

    update_csv(&mut config.name_column);
    update_csv(&mut config.id_column);
    update_csv(&mut config.mark_column);
    update_csv(&mut config.final_mark_column);

    config
        .vertical_postions
        .sort_unstable_by(|a, b| b[1].partial_cmp(&a[1]).unwrap());

    if config.name_postion[1] * 2 <= page_height {
        update_coords(&mut config.name_postion);
        update_coords(&mut config.id_postion);
        let first = config.horizontal_postions[0][1];
        for val in config.horizontal_postions.iter_mut() {
            update_coords(val);
            if val[1] == 0 {
                val[1] = first
            };
        }
        config.horizontal_marks.sort_unstable();
        config
            .horizontal_postions
            .sort_unstable_by(|a, b| a[0].partial_cmp(&b[0]).unwrap());
        update_coords(&mut config.first_marker_postion);
        update_coords(&mut config.second_marker_postion);
        update_coords(&mut config.uel_mark_postion);
        update_coords(&mut config.asu_mark_postion);
        let first = config.vertical_postions[0][0];
        for val in config.vertical_postions.iter_mut() {
            update_coords(val);
            if val[0] == 0 {
                val[0] = first
            };
        }
        config.vertical_postions.reverse();
    };
}

fn pdf_height() -> io::Result<usize> {
    if cfg!(target_os = "windows") {
        Ok(std::string::String::from_utf8(
            Command::new("leafedit.exe")
                .arg("info")
                .arg("page-size")
                .arg("uel_pdf.patched")
                .output()?
                .stdout,
        )
        .unwrap()
        .lines()
        .next()
        .unwrap()
        .split(',')
        .nth(1)
        .unwrap()
        .trim()
        .parse::<f32>()
        .unwrap() as usize)
    } else {
        Ok(std::string::String::from_utf8(
            Command::new("./leafedit")
                .arg("info")
                .arg("page-size")
                .arg("uel_pdf.patched")
                .output()?
                .stdout,
        )
        .unwrap()
        .lines()
        .next()
        .unwrap()
        .split(',')
        .nth(1)
        .unwrap()
        .trim()
        .parse::<f32>()
        .unwrap() as usize)
    }
}

fn expand2() -> io::Result<()> {
    if fs::read_dir("./uel_papers").is_err() {
        fs::create_dir("uel_papers")?;
    }
    if fs::read_dir("./review").is_err() {
        fs::create_dir("review")?;
    }

    if cfg!(target_os = "windows") {
        expand_windows()?;
    };
    if cfg!(target_os = "linux") {
        expand_linux()?;
    };
    if cfg!(target_os = "macos") {
        expand_macos()?;
    };

    Ok(())
}

fn patch_pdf(pdf_path: &PathBuf) -> io::Result<()> {
    if cfg!(target_os = "windows") {
        Command::new("leafedit.exe")
            .arg("patch")
            .arg(pdf_path)
            .arg("uel_pdf.patched")
            .output()?;
    } else if cfg!(target_os = "linux") || cfg!(target_os = "macos") {
        Command::new("./leafedit")
            .arg("patch")
            .arg(pdf_path)
            .arg("uel_pdf.patched")
            .output()?;
    }

    Ok(())
}

fn get_pdf_path(entries: Vec<PathBuf>) -> Option<PathBuf> {
    let mut pdf_path = None;

    for entry in entries {
        if let Some(extention) = entry.extension() {
            if extention == OsStr::new("pdf") {
                pdf_path = Some(entry);
            }
        }
    }

    pdf_path
}

fn expand1() -> io::Result<()> {
    fs::create_dir("csv_files")?;
    fs::write(
        "./uelaur_config.txt",
        include_str!("../include/config.toml"),
    )?;

    Ok(())
}

fn expand_linux() -> Result<(), io::Error> {
    fs::write("leafedit", include_bytes!("../include/linux/leafedit"))?;
    Command::new("chmod").arg("+x").arg("leafedit").output()?;
    Ok(())
}
fn expand_macos() -> Result<(), io::Error> {
    fs::write("leafedit", include_bytes!("../include/macos/leafedit"))?;
    Command::new("spctl")
        .arg("--add")
        .arg("./leafedit")
        .output()?;
    Command::new("chmod").arg("+x").arg("./leafedit").output()?;
    Ok(())
}
fn expand_windows() -> Result<(), io::Error> {
    fs::write(
        "leafedit.exe",
        include_bytes!("../include/windows/leafedit.exe"),
    )?;
    Command::new("cmd")
        .args(["/C", "icacls", "leafedit.exe", "/grant", "*S-1-5-11:(x)"])
        .output()?;
    Ok(())
}

fn ls(path: &str) -> Vec<std::path::PathBuf> {
    fs::read_dir(path)
        .unwrap_or_else(|_| panic!("coundn't read {} dir", path))
        .map(|res| res.map(|e| e.path()))
        .collect::<Result<Vec<_>, io::Error>>()
        .unwrap_or_else(|_| panic!("coundn't create a vec in {}", path))
}

const KEYS: [&str; 18] = [
    "name_column",
    "id_column",
    "mark_column",
    "final_mark_column",
    "name_postion",
    "id_postion",
    "student_field_font_size",
    "horizontal_marks",
    "horizontal_postions",
    "horizontal_feild_font_size",
    "first_marker_postion",
    "second_marker_postion",
    "asu_mark_postion",
    "uel_mark_postion",
    "grade_field_font_size",
    "vertical_marks",
    "vertical_postions",
    "vertical_feild_font_size",
];

#[derive(Debug, Deserialize)]
struct Config {
    name_column: usize,
    id_column: usize,
    mark_column: usize,
    final_mark_column: usize,

    name_postion: [usize; 2],
    id_postion: [usize; 2],
    student_field_font_size: usize,

    horizontal_marks: Vec<usize>,
    horizontal_postions: Vec<[usize; 2]>,
    horizontal_feild_font_size: usize,

    first_marker_postion: [usize; 2],
    second_marker_postion: [usize; 2],
    asu_mark_postion: [usize; 2],
    uel_mark_postion: [usize; 2],
    grade_field_font_size: usize,

    vertical_marks: Vec<[usize; 2]>,
    vertical_postions: Vec<[usize; 2]>,
    vertical_feild_font_size: usize,
}

#[allow(dead_code)]
#[derive(Deserialize)]
struct TestConfig {
    name_column: Option<usize>,
    id_column: Option<usize>,
    mark_column: Option<usize>,
    final_mark_column: Option<usize>,

    name_postion: Option<[usize; 2]>,
    id_postion: Option<[usize; 2]>,
    student_field_font_size: Option<usize>,

    horizontal_marks: Option<Vec<usize>>,
    horizontal_postions: Option<Vec<[usize; 2]>>,
    horizontal_feild_font_size: Option<usize>,

    first_marker_postion: Option<[usize; 2]>,
    second_marker_postion: Option<[usize; 2]>,
    asu_mark_postion: Option<[usize; 2]>,
    uel_mark_postion: Option<[usize; 2]>,
    grade_field_font_size: Option<usize>,

    vertical_marks: Option<Vec<[usize; 2]>>,
    vertical_postions: Option<Vec<[usize; 2]>>,
    vertical_feild_font_size: Option<usize>,
}

#[derive(Debug)]
enum KeyError {
    Repeated(String),
    InCorrect(String),
    Missing(String),
    Typo(String),
    UnExpected(String, &'static str),
}

fn gen_helper_config(invalid_config: String) -> String {
    let mut out = "".to_string();
    for mut line in invalid_config.lines() {
        if let Some(x) = line.find('#') {
            line = line.split_at(x).0.trim();
        } else {
            line = line.trim();
        }
        if !line.is_empty() {
            out += &format!("{}\n", line);
        }
    }
    out += "\n";
    let mut out2 = "".to_string();
    let mut prev = "";
    for line in out.lines() {
        if !prev.is_empty() {
            out2 += &format!("{} ", prev);
        }
        if line.contains('=') {
            out2.pop();
            out2 += "\n";
        }
        prev = line;
    }
    out2.pop();

    let mut lines = out2.lines();
    lines.next();
    let toml_line = lines
        .map(|x| x.split_once('=').unwrap_or(("", "")))
        .collect::<Vec<(&str, &str)>>();

    let mut keys = vec![];
    let mut values = vec![];
    for (key, value) in toml_line {
        keys.push(key.trim());
        values.push(value.trim())
    }

    let mut errors: Vec<KeyError> = vec![];

    let mut sub = 0;
    for (i, key) in keys.clone().iter().enumerate() {
        let i = i - sub;
        if !KEYS.contains(key) {
            let closest = closest(key);
            errors.push(KeyError::UnExpected(
                format!(
                    "# did you mean {}\n{} = {}",
                    closest,
                    keys.remove(i),
                    values.remove(i)
                ),
                closest,
            ));
            sub += 1;
        }
    }

    for key in KEYS {
        match count(&keys, key) {
            0 => errors.push(KeyError::Missing(format!("{} = ", key))),
            1 => (),
            _ => errors.push(remove_all(&mut keys, &mut values, key)),
        }
    }

    let mut sub = 0;
    for i in 0..keys.len() {
        let i = i - sub;
        if toml::from_str::<TestConfig>(&format!("{} = {}", keys[i], values[i])).is_err() {
            errors.push(KeyError::InCorrect(format!(
                "{} = {}",
                keys.remove(i),
                values.remove(i)
            )));
            sub += 1;
        }
    }

    let mut sub = 0;
    for i in 0..errors.len() {
        let i = i - sub;
        let removed = errors.remove(i);
        if let KeyError::UnExpected(key_val, closest) = removed {
            if let Some((j, key)) = find_match(&errors, closest) {
                errors[j] = KeyError::Typo(format!(
                    "# did you mean {}\n{}",
                    key,
                    key_val.lines().nth(1).unwrap()
                ));
                sub += 1;
            } else {
                errors.insert(i, KeyError::UnExpected(key_val, closest));
            }
        } else {
            errors.insert(i, removed);
        }
    }

    create_helper(errors, &keys, &values)
}

fn create_helper(errors: Vec<KeyError>, keys: &[&str], values: &[&str]) -> String {
    let mut helper_config = "".to_owned();

    let mut typo = "".to_string();
    let mut repeated = "".to_string();
    let mut missing = "".to_string();
    let mut unexpected = "".to_string();
    let mut incorrect = "".to_string();

    for item in errors {
        match item {
            KeyError::Repeated(err) => repeated += &format!("{}\n\n", err),
            KeyError::InCorrect(err) => incorrect += &format!("{}\n\n", err),
            KeyError::Missing(err) => missing += &format!("{}\n\n", err),
            KeyError::Typo(err) => typo += &format!("{}\n\n", err),
            KeyError::UnExpected(err, _) => unexpected += &format!("{}\n\n", err),
        }
    }

    typo.pop();
    typo.pop();
    repeated.pop();
    repeated.pop();
    missing.pop();
    missing.pop();
    unexpected.pop();
    unexpected.pop();
    incorrect.pop();
    incorrect.pop();

    if let Some(mut err) = error_text(typo, "## misspelled option".to_owned()) {
        err.push_str("\n\n\n");
        helper_config.push_str(&err);
    }

    if let Some(mut err) = error_text(repeated, "## repeated option".to_owned()) {
        err.push_str("\n\n\n");
        helper_config.push_str(&err);
    }

    if let Some(mut err) = error_text(missing, "## missing option".to_owned()) {
        err.push_str("\n\n\n");
        helper_config.push_str(&err);
    }

    if let Some(mut err) = error_text(unexpected, "## un-expected option".to_owned()) {
        err.push_str("\n\n\n");
        helper_config.push_str(&err);
    }

    if let Some(mut err) = error_text(incorrect, "## in-correct option value".to_owned()) {
        err.push_str("\n\n\n");
        helper_config.push_str(&err);
    }

    helper_config.push_str("# Correct\n");
    for i in 0..keys.len() {
        helper_config.push_str(&format!("{} = {}\n", keys[i], values[i]));
    }

    helper_config
}

fn error_text(str1: String, mut str2: String) -> Option<String> {
    if !str1.is_empty() {
        if str1.matches('\n').count() - str1.matches('#').count() > 1 {
            str2.push('s');
            str2.push('\n');
            str2.push_str(&str1);
        } else {
            str2.push('\n');
            str2.push_str(&str1);
        }
        Some(str2)
    } else {
        None
    }
}

fn find_match<'a>(errors: &'a [KeyError], eq: &str) -> Option<(usize, &'a str)> {
    let mut dis = None;
    let mut ret = None;
    for (i, err) in errors.iter().enumerate() {
        if let KeyError::Missing(key) = err {
            let distance = edit_distance(key, eq);
            if distance < dis.unwrap_or(usize::MAX) {
                dis = Some(distance);
                if dis.unwrap() < 7 {
                    ret = Some((i, key.as_str().get(0..key.len() - 3).unwrap()));
                }
            }
        }
    }
    ret
}

fn count(vec: &[&str], eq: &str) -> usize {
    let vec_clone = vec.to_owned();
    vec_clone.iter().filter(|x| **x == eq).count()
}

fn find_all(vec: &[&str], eq: &str) -> Vec<usize> {
    let mut index: Vec<usize> = vec![];
    let mut sub = 0;
    for (i, key) in vec.iter().enumerate() {
        let i = i - sub;
        if *key == eq {
            index.push(i);
            sub += 1;
        }
    }
    index
}

fn remove_all(vec1: &mut Vec<&str>, vec2: &mut Vec<&str>, eq: &str) -> KeyError {
    let mut all = "".to_owned();
    for i in find_all(vec1, eq) {
        all.push_str("# ");
        all.push_str(vec1.remove(i));
        all.push_str(" = ");
        all.push_str(vec2.remove(i));
        all.push('\n');
    }
    KeyError::Repeated(format!("{}{} = ", all, eq))
}

fn closest(eq: &str) -> &'static str {
    let mut current = "";
    for key in KEYS {
        if edit_distance(key, eq) < edit_distance(current, eq) {
            current = key;
        }
    }
    current
}
const ANIMITION: [&str; 14] = [
    "\x1b[32m[------]\x1b[0m",
    "\x1b[32m[------]\x1b[0m",
    "\x1b[32m[\x1b[0mo\x1b[32m-----]\x1b[0m",
    "\x1b[32m[-\x1b[0mo\x1b[32m----]\x1b[0m",
    "\x1b[32m[--\x1b[0mo\x1b[32m---]\x1b[0m",
    "\x1b[32m[---\x1b[0mo\x1b[32m--]\x1b[0m",
    "\x1b[32m[---\x1b[0mo\x1b[32m--]\x1b[0m",
    "\x1b[32m[\x1b[33mc\x1b[32m--\x1b[0mo\x1b[32m--]\x1b[0m",
    "\x1b[32m[-\x1b[33mc\x1b[32m-\x1b[0mo\x1b[32m--]\x1b[0m",
    "\x1b[32m[--\x1b[33mc\x1b[0mo\x1b[32m--]\x1b[0m",
    "\x1b[32m[--\x1b[33mC\x1b[0mo\x1b[32m--]\x1b[0m",
    "\x1b[32m[---\x1b[33mC\x1b[32m--]\x1b[0m",
    "\x1b[32m[----\x1b[33mc\x1b[32m-]\x1b[0m",
    "\x1b[32m[-----\x1b[33mc\x1b[32m]\x1b[0m",
];

fn init_ui(recv: mpsc::Receiver<Action>, finish: mpsc::Sender<u8>) -> thread::JoinHandle<()> {
    let mut ansi = match enable_ansi_support::enable_ansi_support() {
        Ok(_) => Ansi::new(false, ANIMITION.to_vec()),
        Err(_) => Ansi::new(true, vec![]),
    };

    let duration = time::Duration::new(0, 120_000_000);

    thread::spawn(move || {
        ansi.intro();
        loop {
            if let Ok(action) = recv.try_recv() {
                match action {
                    Action::Msg(msg) => ansi.display(msg),
                    Action::Update(msg) => ansi.next(&msg),
                    Action::Quit => {
                        ansi.clean();
                        break;
                    }
                    Action::Panic => {
                        ansi.panic();
                        break;
                    }
                };
            } else {
                ansi.next("");
            }
            thread::sleep(duration);
        }
        finish.send(0).unwrap();
    })
}

enum Action {
    Msg(Message),
    Update(String),
    Panic,
    Quit,
}

#[allow(dead_code)]
enum Message {
    Warn(String),
    Fail(String),
    Succ(String),
    Fatal(String),
}

struct Ansi {
    no_ansi: bool,
    lenth: usize,
    index: usize,
    loop_chars: Vec<&'static str>,
}

impl Ansi {
    fn new(no_ansi: bool, loop_chars: Vec<&'static str>) -> Ansi {
        Ansi {
            no_ansi,
            lenth: 0,
            index: 0,
            loop_chars,
        }
    }

    fn clean(&self) {
        if self.no_ansi {
            println!("\ndone !!!\n");
        } else {
            println!("\n\x1b[1A{}\ndone !!!", " ".repeat(10 + self.lenth));
        }
    }

    fn panic(&self) {
        if self.no_ansi {
            println!("\n\n");
        } else {
            println!("\n\x1b[1A{}\n", " ".repeat(10 + self.lenth));
        }
    }

    fn next(&mut self, msg: &str) {
        if self.no_ansi {
            if !msg.is_empty() {
                println!("{}\n", msg);
            }
        } else {
            self.index += 1;
            self.index %= self.loop_chars.len();

            if !msg.is_empty() {
                for _ in 0..=self.lenth + 1 {
                    print!(" ");
                }
                self.lenth = msg.len();
            }
            print!("\n\x1b[1A");

            print!(" {} {}", self.loop_chars[self.index], msg);
        }
    }

    fn display(&self, msg: Message) {
        if self.no_ansi {
            match msg {
                Message::Warn(m) => println!("Waring: {}\n", m),
                Message::Fail(m) => println!("Failure: {}\n", m),
                Message::Succ(m) => println!("Success: {}\n", m),
                Message::Fatal(m) => println!("FATAL ERROR: {}\n", m.to_uppercase()),
            }
        } else {
            print!("\n\x1b[1A{}\n\x1b[2A", " ".repeat(10 + self.lenth));
            match msg {
                Message::Warn(m) => println!("\n\x1b[33mwaring\x1b[0m: {}", m),
                Message::Fail(m) => println!("\n\x1b[31mfailure\x1b[0m: {}", m),
                Message::Succ(m) => println!("\n\x1b[32msuccess\x1b[0m: {}", m),
                Message::Fatal(m) => println!(
                    "\n\x1b[31mFATAL ERROR\x1b[0m: \x1b[31m{}\x1b[0m",
                    m.to_uppercase()
                ),
            }
        }
    }

    fn intro(&self) {
        let uelaur = [
            ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ',
            ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ',
            ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ',
            ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ',
            ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', '▄', '▄', ' ', ' ', ' ', ' ',
            ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ',
            ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ',
            ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', '▀', '█', '█',
            '█', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ',
            ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ',
            ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ',
            ' ', ' ', ' ', ' ', '█', '█', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ',
            ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ',
            ' ', ' ', ' ', ' ', '▀', '█', '█', '█', ' ', ' ', '▀', '█', '█', '█', ' ', ' ', ' ',
            '▄', '▄', '█', '▀', '█', '█', ' ', ' ', ' ', '█', '█', ' ', ' ', ' ', '▄', '█', '▀',
            '█', '█', '▄', ' ', '▀', '█', '█', '█', ' ', ' ', '▀', '█', '█', '█', ' ', ' ', '▀',
            '█', '█', '█', '▄', '█', '█', '█', ' ', ' ', ' ', ' ', '█', '█', ' ', ' ', ' ', ' ',
            '█', '█', ' ', ' ', '▄', '█', '▀', ' ', ' ', ' ', '█', '█', ' ', ' ', '█', '█', ' ',
            ' ', '█', '█', ' ', ' ', ' ', '█', '█', ' ', ' ', ' ', '█', '█', ' ', ' ', ' ', ' ',
            '█', '█', ' ', ' ', ' ', ' ', '█', '█', '▀', ' ', '▀', '▀', ' ', ' ', ' ', ' ', '█',
            '█', ' ', ' ', ' ', ' ', '█', '█', ' ', ' ', '█', '█', '▀', '▀', '▀', '▀', '▀', '▀',
            ' ', ' ', '█', '█', ' ', ' ', ' ', '▄', '█', '█', '█', '█', '█', ' ', ' ', ' ', '█',
            '█', ' ', ' ', ' ', ' ', '█', '█', ' ', ' ', ' ', ' ', '█', '█', ' ', ' ', ' ', ' ',
            ' ', ' ', ' ', ' ', '█', '█', ' ', ' ', ' ', ' ', '█', '█', ' ', ' ', '█', '█', '▄',
            ' ', ' ', ' ', ' ', '▄', ' ', ' ', '█', '█', ' ', ' ', '█', '█', ' ', ' ', ' ', '█',
            '█', ' ', ' ', ' ', '█', '█', ' ', ' ', ' ', ' ', '█', '█', ' ', ' ', ' ', ' ', '█',
            '█', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', '▀', '█', '█', '█', '█', '▀', '█', '█',
            '█', '▄', ' ', '▀', '█', '█', '█', '█', '█', '▀', '▄', '█', '█', '█', '█', '▄', ' ',
            '█', '█', '█', '█', '▀', '█', '█', '▄', ' ', '▀', '█', '█', '█', '█', '▀', '█', '█',
            '█', ' ', '▄', '█', '█', '█', '█', '▄', ' ', ' ', ' ',
        ];
        if self.no_ansi {
            println!("\n UEL_AU\n")
        } else {
            for _ in 0..10 {
                println!();
            }
            print!("\n\x1b[10A  ");
            for i in 1..56 {
                for j in 0..9 {
                    print!("{}\x1b[1B\x1b[1D", uelaur[i + 56 * j]);
                    std::io::stdout().flush().ok();
                    thread::sleep(time::Duration::new(0, 6_000_000));
                }
                print!("\n\x1b[10A\x1b[{}C", i + 1);
                thread::sleep(time::Duration::new(0, 10_000_000));
            }
            for _ in 0..10 {
                println!();
            }
        }
    }
}
