use std::{fs, io::Write, path::PathBuf};
use tch::{Kind, Tensor};
use memmap2::MmapOptions;
use std::fs::File;

fn slugify(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        if c.is_ascii_alphanumeric() { out.push(c.to_ascii_lowercase()); } else { out.push('_'); }
    }
    out
}

fn seed_dir() -> PathBuf { PathBuf::from("seeds") }

fn seed_path(subject: &str) -> PathBuf {
    let mut p = seed_dir();
    p.push(format!("{}.bin", slugify(subject)));
    p
}

pub fn save_seed_matrix(subject: &str, v: &Tensor) -> anyhow::Result<()> {
    let _ = fs::create_dir_all(seed_dir());
    let path = seed_path(subject);
    let v = v.to_device(tch::Device::Cpu).to_kind(Kind::Float);
    let mut f = fs::File::create(path)?;
    for i in 0..9 {
        let x = v.double_value(&[0, i]) as f32;
        f.write_all(&x.to_le_bytes())?;
    }
    Ok(())
}

pub fn load_seed_matrix(subject: &str) -> anyhow::Result<Option<Tensor>> {
    let path = seed_path(subject);
    if !path.exists() { return Ok(None); }
    let file = File::open(path)?;
    let mmap = unsafe { MmapOptions::new().len(9 * 4).map(&file)? };
    if mmap.len() < 9 * 4 { return Ok(None); }
    let mut vs = Vec::with_capacity(9);
    for i in 0..9 {
        let off = i * 4;
        let mut arr = [0u8; 4];
        arr.copy_from_slice(&mmap[off..off + 4]);
        vs.push(f32::from_le_bytes(arr));
    }
    let t = Tensor::of_slice(&vs).view([1, 9]).softmax(-1, Kind::Float);
    Ok(Some(t))
}
