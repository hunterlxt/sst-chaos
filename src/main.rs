use rand::Rng;
use std::fs;
use std::os::unix::fs::FileExt;
use std::path;
use std::path::PathBuf;
use std::process::Command;

const DAMAGE_INDEX: u64 = 1 * 1024 * 1024;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    assert_eq!(
        args.len(),
        3,
        "usage: <cmd> <ldb_path> <rocksdb_manifest_path> NOTE: use absolute path"
    );
    let mut db_path = PathBuf::from(&args[2]);
    assert!(db_path.exists());
    assert!(db_path.is_file());
    db_path.pop();

    let output = Command::new(&args[1])
        .args(["manifest_dump", &format!("--path={}", &args[2])])
        .output()
        .expect("failed to execute process");

    let mut out_string = std::string::String::from_utf8_lossy(&output.stdout).to_string();

    let index = out_string.find("--- level 7").unwrap();
    out_string.truncate(index);
    let index = out_string.find("--- level 1").unwrap();
    out_string.drain(..index);
    let out = out_string.trim().to_owned();

    let mut ssts = vec![];
    for s in out.split_whitespace() {
        if let Some(result) = check_str(s) {
            ssts.push(result);
        }
    }
    if ssts.len() < 2 {
        println!("can't find enough ssts");
        std::process::exit(-1);
    }
    let mut ssts: Vec<path::PathBuf> = ssts
        .into_iter()
        .map(|mut name| {
            name.push_str(".sst");
            db_path.join(name)
        })
        .collect();

    ssts.retain(|p| check_size(p));

    let mut rng = rand::thread_rng();
    let s1 = rng.gen_range(0..ssts.len());
    let mut s2 = rng.gen_range(0..ssts.len());
    while s2 == s1 {
        s2 = rng.gen_range(0..ssts.len());
    }

    println!("ready to damage: {:?} {:?}", &ssts[s1], &ssts[s2]);

    damage_sst(&ssts[s1]);
    damage_sst(&ssts[s2]);

    println!("inject finished");
}

fn check_str(s: &str) -> Option<String> {
    let mut counting_sst_mark = true;
    let mut sst_num_count = 0;
    let mut post_count = 0;
    for (i, c) in s.char_indices() {
        if counting_sst_mark {
            if c.is_digit(10) {
                sst_num_count += 1;
                if sst_num_count > 6 {
                    return None;
                }
            } else if c == ':' {
                if i == 0 {
                    return None;
                }
                counting_sst_mark = false;
            } else {
                return None;
            }
        } else {
            if c.is_digit(10) {
                post_count += 1;
            } else if c == '[' {
                if post_count == 0 {
                    return None;
                }
                break;
            } else {
                return None;
            }
        }
    }
    let sst = s.split_once(':')?.0;
    Some(format!("{:0>6}", sst))
}

fn check_size(path: &path::Path) -> bool {
    let meta = fs::metadata(&path);
    if let Ok(meta) = meta {
        if meta.len() > 1024 * 1024 * 4 {
            return true;
        }
    }
    false
}

fn damage_sst(path: &path::Path) {
    assert!(path.exists());
    assert!(path.is_file());
    let file = fs::OpenOptions::new()
        .read(true)
        .write(true)
        .open(path)
        .unwrap();

    let buf = &[0];
    file.write_at(buf, DAMAGE_INDEX).unwrap();
    file.write_at(buf, DAMAGE_INDEX * 2).unwrap();
    file.sync_all().unwrap();
}
