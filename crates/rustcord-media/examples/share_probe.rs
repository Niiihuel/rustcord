//! What sharing a screen costs before the encoder, by the same road a call
//! takes: the portal's dialog (or the choice a token kept in a file names),
//! then, once a second, what the capture delivered and where its time went.
//!
//! `cargo run --release -p rustcord-media --example share_probe -- [seconds] [token file] [height] [fps] [encode]`
//! (10 seconds, no file, 720 and 30 by default). The token file is read
//! before and written after, so a second run needs no dialog. With a fifth
//! word the frames go to the encoder a call uses (the GPU's, then
//! OpenH264), in this process as in the application. Linux only.

#[cfg(target_os = "linux")]
fn main() {
    use std::time::{Duration, Instant};

    use rustcord_media::video::{Video, VideoState};
    use rustcord_model::ShareKind;

    let mut arguments = std::env::args().skip(1);
    let seconds: u64 = arguments.next().and_then(|n| n.parse().ok()).unwrap_or(10);
    let token_file = arguments.next();
    let height: u32 = arguments.next().and_then(|n| n.parse().ok()).unwrap_or(720);
    let fps: u32 = arguments.next().and_then(|n| n.parse().ok()).unwrap_or(30);
    let restore = token_file
        .as_ref()
        .and_then(|file| std::fs::read_to_string(file).ok())
        .map(|token| token.trim().to_owned())
        .filter(|token| !token.is_empty());
    println!(
        "asking the system ({})",
        if restore.is_some() { "token" } else { "dialog" }
    );
    let size = ((height * 16).div_ceil(9) & !1, height);
    let video = Video::share(None, restore, ShareKind::Any, size, fps);
    let encode = arguments.next().is_some_and(|word| word == "encode");
    let mut take = video.frames();
    let asked = Instant::now();
    loop {
        match video.snapshot().state {
            VideoState::Running => break,
            VideoState::Ended(reason) => return println!("ended: {reason:?}"),
            VideoState::Starting if asked.elapsed() > Duration::from_secs(120) => {
                return println!("nobody chose a screen");
            }
            VideoState::Starting => std::thread::sleep(Duration::from_millis(50)),
        }
    }
    if let (Some(file), Some(token)) = (&token_file, video.snapshot().restore_token) {
        let _ = std::fs::write(file, token);
    }
    let mut before = video.capture_stats();
    // The call's encoder, its units let go of; or nothing but taking frames.
    let encoding = encode.then(|| {
        use rustcord_media::video::encode::{Encoding, Settings, HARDWARE, MIN_BITRATE};
        let settings = Settings {
            fps,
            bitrate: 3_500_000,
            min_bitrate: MIN_BITRATE,
            max_bitrate: 3_500_000,
            camera: false,
            engines: HARDWARE,
        };
        Encoding::start(settings, video.frames(), || true, |_| true, || None)
    });
    for _ in 0..seconds {
        let second = Instant::now();
        while second.elapsed() < Duration::from_secs(1) {
            if encoding.is_none() {
                let _ = take();
            }
            let _ = video.take_frame();
            std::thread::sleep(Duration::from_millis(5));
        }
        let now = video.capture_stats();
        let kept = now.kept - before.kept;
        let each = |micros: u64, then: u64| (micros - then) as f64 / 1000.0 / kept.max(1) as f64;
        println!(
            "{}x{} {} {}: delivered {}, kept {}; {:.1} ms converting ({:.1} mapping)",
            now.size.0,
            now.size.1,
            now.format,
            now.memory,
            now.delivered - before.delivered,
            kept,
            each(now.converting_micros, before.converting_micros),
            each(now.mapping_micros, before.mapping_micros),
        );
        before = now;
    }
    if let Some(encoding) = encoding {
        println!("encoder: {:?}", encoding.stats());
    }
}

#[cfg(not(target_os = "linux"))]
fn main() {}
