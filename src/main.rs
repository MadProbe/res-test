#![feature(fs_try_exists, core_intrinsics, new_uninit)]
#![allow(internal_features)]
use std::arch::x86_64::*;
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;
use std::time::Instant;
use cfg_if::cfg_if;
use sysinfo::Disks;
const BUFFER_SIZE_POWER: u32 = 12;

fn read_test(path: &mut String, file_name: &mut String) -> std::io::Result<()> {
    unsafe {
        let mut shit_times = 0;
        let start_time = Instant::now();
        let mut file = File::open(path.to_owned() + "/" + file_name.as_str())?;
        let space = std::fs::metadata(path.to_owned() + "/" + file_name.as_str())?.len();
        let mut last_speeds: Vec<f64> = vec![];
        let mut buffer = [0u8; 2usize.pow(BUFFER_SIZE_POWER)];
        let mut xmm0 = _mm_set_epi8(0x2f, 0x2b, 0x29, 0x25, 0x1f, 0x1d, 0x17, 0x13, 0x11, 0x0D, 0x0B, 0x07, 0x05, 0x03, 0x02, 0x01);
        let mut xmm1 = _mm_setzero_si128();
        let mut xmm2 = _mm_add_epi64(xmm1, xmm0);

        let mut start: Instant = Instant::now();
        let mut bufs = 0u64;
        for its in 0u64..(space / 2u64.pow(BUFFER_SIZE_POWER)) {
            if std::intrinsics::unlikely(file.read(&mut buffer)? != buffer.len()) {
                panic!("THIS SHIT DIDN'T READ ALL THE WAY!!! THIS MAY BE A RUST OR YOUR OS BUG AND NOT A PROBLEM IN FLASH DRIVE!!!")
            }
            for offset in (0usize..(2usize.pow(BUFFER_SIZE_POWER) / 64usize)).map(|a| a * 64) {
                let xmm3 = _mm_aesenc_si128(xmm1, xmm0);
                let xmm4 = _mm_aesenc_si128(xmm2, xmm0);
                let xmm5 = _mm_aesenc_si128(xmm3, xmm0);
                let xmm6 = _mm_aesdec_si128(xmm3, xmm0);
                let xmm7 = _mm_aesenc_si128(xmm4, xmm0);
                let xmm8 = _mm_aesdec_si128(xmm4, xmm0);
                let xmm9 = _mm_cmpeq_epi8(xmm5, _mm_loadu_si128(buffer.as_ptr().wrapping_byte_add(offset) as *mut _));
                let xmm10 = _mm_cmpeq_epi8(xmm6, _mm_loadu_si128(buffer.as_ptr().wrapping_byte_add(offset + 16) as *mut _));
                let xmm11 = _mm_cmpeq_epi8(xmm7, _mm_loadu_si128(buffer.as_ptr().wrapping_byte_add(offset + 32) as *mut _));
                let xmm12 = _mm_cmpeq_epi8(xmm8, _mm_loadu_si128(buffer.as_ptr().wrapping_byte_add(offset + 48) as *mut _));
                let xmm13 = _mm_cmpeq_epi8(_mm_and_si128(_mm_and_si128(xmm9, xmm10), _mm_and_si128(xmm11, xmm12)), _mm_setzero_si128());
                if std::intrinsics::unlikely(_mm_testz_si128(xmm13, xmm13) == 0) { // Something's not right... 
                    for (i, (xmm_1, xmm_2)) in [xmm9, xmm10, xmm11, xmm12].into_iter().zip([xmm5, xmm6, xmm7, xmm8]).enumerate() {
                        if _mm_testz_si128(xmm_1, xmm_1) == 1 {
                            println!(
                                "[{:#020X}] [{:}] != [{:}]",
                                its * 2u64.pow(BUFFER_SIZE_POWER) + (offset as u64) + (i as u64) * 16,
                                std::intrinsics::transmute_unchecked::<__m128i, [u8; 16]>(_mm_loadu_si128(buffer.as_mut_ptr().wrapping_byte_add(offset + 16 * i) as *mut _)).map(|a| format!("0x{:02X}", a)).join(", "),
                                std::intrinsics::transmute_unchecked::<__m128i, [u8; 16]>(xmm_2).map(|a| format!("0x{:02X}", a)).join(", "),
                            );
                            shit_times += 1;
                        }
                        if shit_times == 512 {
                            panic!("FUCK!!! THAT DIDN'T WORK!!!")
                        }
                    }
                }
                xmm1 = _mm_add_epi64(xmm1, xmm0);
                xmm2 = _mm_add_epi64(xmm2, xmm0);
                xmm0 = _mm_aesenc_si128(xmm0, xmm0);
            }
            let mut secs = Instant::now().duration_since(start).as_secs_f64();
            if secs >= 1f64 {
                file.flush()?;
                secs = Instant::now().duration_since(start).as_secs_f64();
                let speed = ((its - bufs) as f64) * 2f64.powf((BUFFER_SIZE_POWER as i32 - 20) as f64) / secs;
                last_speeds.push(speed);
                if last_speeds.len() > 1024 {
                    last_speeds.remove(0);
                }
                println!(
                    "[{:#?}] Read bytes: {:.3}mb, speed: {:.3}mb/s, average speed: {:.3}mb/s",
                    Instant::now().duration_since(start_time),
                    (its as f64) * 2f64.powf((BUFFER_SIZE_POWER as i32 - 20) as f64),
                    speed,
                    last_speeds.iter().map(|&x| x).reduce(|a, x| std::intrinsics::fadd_fast(a, x)).unwrap() / (last_speeds.len() as f64)
                );
                bufs = its;
                start = Instant::now();
            }
        }
        println!("Finished in {:#?}!", Instant::now().duration_since(start_time));
        Ok(())
    }
}

fn write_test(path: &mut String, file_name: &mut String) -> std::io::Result<()> {
    unsafe {
        let start_time = Instant::now();
        if std::fs::try_exists(path.to_owned() + "/" + file_name.as_str())? {
            std::fs::remove_file(path.to_owned() + "/" + file_name.as_str())?;
        }
        let free_space = Disks::new_with_refreshed_list()
            .list()
            .into_iter()
            .find(|disk| disk.mount_point() == Path::new(path.as_str()))
            .unwrap()
            .available_space();
        let mut file = File::create_new(path.to_owned() + "/" + file_name.as_str())?;
        file.set_len(free_space)?;
        let mut last_speeds: Vec<f64> = vec![];
        let mut buffer = [0u8; 2usize.pow(BUFFER_SIZE_POWER)];
        let mut xmm0 = _mm_set_epi8(0x2f, 0x2b, 0x29, 0x25, 0x1f, 0x1d, 0x17, 0x13, 0x11, 0x0D, 0x0B, 0x07, 0x05, 0x03, 0x02, 0x01);
        let mut xmm1 = _mm_setzero_si128();
        let mut xmm2 = _mm_add_epi64(xmm1, xmm0);

        let mut start: Instant = Instant::now();
        let mut bufs = 0u64;

        for its in 0u64..free_space / 2u64.pow(BUFFER_SIZE_POWER) {
            for offset in (0usize..(2usize.pow(BUFFER_SIZE_POWER) / 64usize)).map(|a| a * 64) {
                let xmm3 = _mm_aesenc_si128(xmm1, xmm0);
                let xmm4 = _mm_aesenc_si128(xmm2, xmm0);
                let xmm5 = _mm_aesenc_si128(xmm3, xmm0);
                let xmm6 = _mm_aesdec_si128(xmm3, xmm0);
                let xmm7 = _mm_aesenc_si128(xmm4, xmm0);
                let xmm8 = _mm_aesdec_si128(xmm4, xmm0);
                _mm_storeu_si128(buffer.as_mut_ptr().wrapping_byte_add(offset) as *mut _, xmm5);
                _mm_storeu_si128(buffer.as_mut_ptr().wrapping_byte_add(offset + 16) as *mut _, xmm6);
                _mm_storeu_si128(buffer.as_mut_ptr().wrapping_byte_add(offset + 32) as *mut _, xmm7);
                _mm_storeu_si128(buffer.as_mut_ptr().wrapping_byte_add(offset + 48) as *mut _, xmm8);
                xmm1 = _mm_add_epi64(xmm1, xmm0);
                xmm2 = _mm_add_epi64(xmm2, xmm0);
                xmm0 = _mm_aesenc_si128(xmm0, xmm0);
            }
            if std::intrinsics::unlikely(file.write(&buffer)? != buffer.len()) {
                panic!("THIS SHIT DIDN'T WRITE ALL THE WAY!!! THIS MAY BE A RUST OR YOUR OS BUG AND NOT A PROBLEM IN FLASH DRIVE!!!")
            }
            let mut secs = Instant::now().duration_since(start).as_secs_f64();
            if secs >= 1f64 {
                secs = Instant::now().duration_since(start).as_secs_f64();
                let speed = ((its - bufs) as f64) * 2f64.powf((BUFFER_SIZE_POWER as i32 - 20) as f64) / secs;
                last_speeds.push(speed);
                if last_speeds.len() > 1024 {
                    last_speeds.remove(0);
                }
                println!(
                    "[{:#?}] Written bytes: {:.3}mb, speed: {:.3}mb/s, average speed: {:.3}mb/s",
                    Instant::now().duration_since(start_time),
                    (its as f64) * 2f64.powf((BUFFER_SIZE_POWER as i32 - 20) as f64),
                    speed,
                    last_speeds.iter().map(|&x| x).reduce(|a, x| std::intrinsics::fadd_fast(a, x)).unwrap() / (last_speeds.len() as f64)
                );
                bufs = its;
                start = Instant::now();
            }
        }
        file.flush()?;
        println!("Finished in {:#?}!", Instant::now().duration_since(start_time));
    }
    Ok(())
}

#[inline(never)]
pub fn find_longest_running_number() -> u32 {
    let mut set = unsafe { Box::<[bool; 2usize.pow(32)]>::new_uninit().assume_init() };
    (2u32..10000u32).filter(|&a| a & 1 == 1).map(|a| {
        let start = Instant::now();
        for offset in (0usize..set.len() / 256).map(|a| a * 256) {
            unsafe {
                cfg_if! {
                    if #[cfg(target_feature="avx")] {
                        let zeroes = _mm256_setzero_si256();
                        _mm256_storeu_si256(set.as_mut_ptr().wrapping_byte_add(offset) as *mut _, zeroes);
                        _mm256_storeu_si256(set.as_mut_ptr().wrapping_byte_add(offset + 32) as *mut _, zeroes);
                        _mm256_storeu_si256(set.as_mut_ptr().wrapping_byte_add(offset + 64) as *mut _, zeroes);
                        _mm256_storeu_si256(set.as_mut_ptr().wrapping_byte_add(offset + 96) as *mut _, zeroes);
                        _mm256_storeu_si256(set.as_mut_ptr().wrapping_byte_add(offset + 128) as *mut _, zeroes);
                        _mm256_storeu_si256(set.as_mut_ptr().wrapping_byte_add(offset + 160) as *mut _, zeroes);
                        _mm256_storeu_si256(set.as_mut_ptr().wrapping_byte_add(offset + 192) as *mut _, zeroes);
                        _mm256_storeu_si256(set.as_mut_ptr().wrapping_byte_add(offset + 224) as *mut _, zeroes);
                    } else {
                        let zeroes = _mm_setzero_si128();
                        _mm_storeu_si128(set.as_mut_ptr().wrapping_byte_add(offset) as *mut _, zeroes);
                        _mm_storeu_si128(set.as_mut_ptr().wrapping_byte_add(offset + 16) as *mut _, zeroes);
                        _mm_storeu_si128(set.as_mut_ptr().wrapping_byte_add(offset + 32) as *mut _, zeroes);
                        _mm_storeu_si128(set.as_mut_ptr().wrapping_byte_add(offset + 48) as *mut _, zeroes);
                        _mm_storeu_si128(set.as_mut_ptr().wrapping_byte_add(offset + 64) as *mut _, zeroes);
                        _mm_storeu_si128(set.as_mut_ptr().wrapping_byte_add(offset + 80) as *mut _, zeroes);
                        _mm_storeu_si128(set.as_mut_ptr().wrapping_byte_add(offset + 96) as *mut _, zeroes);
                        _mm_storeu_si128(set.as_mut_ptr().wrapping_byte_add(offset + 112) as *mut _, zeroes);
                        _mm_storeu_si128(set.as_mut_ptr().wrapping_byte_add(offset + 128) as *mut _, zeroes);
                        _mm_storeu_si128(set.as_mut_ptr().wrapping_byte_add(offset + 144) as *mut _, zeroes);
                        _mm_storeu_si128(set.as_mut_ptr().wrapping_byte_add(offset + 160) as *mut _, zeroes);
                        _mm_storeu_si128(set.as_mut_ptr().wrapping_byte_add(offset + 176) as *mut _, zeroes);
                        _mm_storeu_si128(set.as_mut_ptr().wrapping_byte_add(offset + 192) as *mut _, zeroes);
                        _mm_storeu_si128(set.as_mut_ptr().wrapping_byte_add(offset + 208) as *mut _, zeroes);
                        _mm_storeu_si128(set.as_mut_ptr().wrapping_byte_add(offset + 224) as *mut _, zeroes);
                        _mm_storeu_si128(set.as_mut_ptr().wrapping_byte_add(offset + 240) as *mut _, zeroes);
                    }
                }
            }
        }
        println!("{:#?}", start.elapsed());
        let mut i = a;
        let mut r = 0u32;
        while !set[i as usize] {
            set[i as usize] = true;
            i *= a;
            r += 1;
        }
        println!("{:}, {:}", a, r);
        (a, r)

    }).max_by_key(|t| t.1).unwrap().0
}

fn main() -> std::io::Result<()> {
    if !is_x86_feature_detected!("sse4.1") || !is_x86_feature_detected!("aes") {
        panic!("Your CPU doesn't support SSE4.1 & AES-NI instructions, you dummy dumb, you need a better CPU LOL and not this ancient bullshit!");
    }
    let mut i = 0;
    let mut path = String::from("res-test-stuff.bin");
    let mut path_replace = false;
    let mut drive = String::from("");
    let mut drive_replace = false;
    for arg in std::env::args() {
        if drive_replace {
            drive = arg;
            drive_replace = false;
            continue;
        }
        if path_replace {
            path = arg;
            path_replace = false;
            continue;
        }
        if arg == "--path" {
            path_replace = true;
        } else if arg == "--drive" {
            drive_replace = true;
        } else if arg == "--read-test" {
            i = 1;
        } else if arg == "--write-test" {
            i = 2;
        } else if arg == "--full-test" {
            i = 3;
        } else if arg == "--flrn" {
            i = 4;
        }
    }
    if i == 1 { // read_test
        return read_test(&mut drive, &mut path);
    } else if i == 2 { // write_test
        return write_test(&mut drive, &mut path);
    } else if i == 3 { // full test
        write_test(&mut drive, &mut path)?;
        return read_test(&mut drive, &mut path);
    } else if i == 4 { // find_longest_running_number
        println!("{:}", find_longest_running_number());
        return Ok(());
    }
    panic!("Hey, we didn't do anything! Try specifing drive with --drive CLI argument lie this: `--drive G:` and try using --read-test argument to the CLI if you want to test a flash drive for data coherency or pass --write-test argument to the CLI if you want to test flash drive if it can write data to all the space it advertises it has or you can do --full-test if you want to do both");
}
