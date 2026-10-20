//! Public deterministic local-byte demo. No internet, payments, proofs or consensus.
#[allow(dead_code)]
#[path = "support/byte_budget.rs"]
mod budget;
#[allow(dead_code)]
#[path = "support/observation_fixture.rs"]
mod fixture;
#[allow(dead_code)]
#[path = "support/retrieval_observation.rs"]
mod retrieval;

#[cfg(unix)]
mod demo {
    use super::{budget, fixture, retrieval};
    use std::{
        error::Error,
        fs::{self, OpenOptions},
        io::{self, Read, Write},
        os::unix::net::UnixStream,
        path::Path,
        sync::{
            atomic::{AtomicBool, AtomicUsize, Ordering},
            Arc,
        },
        time::{Duration, Instant},
    };

    type Failure = Box<dyn Error + Send + Sync>;

    fn require(value: bool, reason: &'static str) -> Result<(), Failure> {
        if value {
            Ok(())
        } else {
            Err(io::Error::other(reason).into())
        }
    }

    fn wave(
        output: &Path,
        name: &str,
        wave_number: u8,
        sources: [&Path; 2],
        meters: &[Arc<budget::Meter>; 2],
        manifest: &retrieval::Manifest,
        start: Instant,
    ) -> Result<(usize, usize), Failure> {
        let gate = Arc::new(AtomicBool::new(false));
        let ready = Arc::new(AtomicUsize::new(0));
        let mut requests = [fixture::request(manifest, 0), fixture::request(manifest, 1)];
        for request in &mut requests {
            request.interval = wave_number;
            request.nonce[0] = request.nonce[0].wrapping_add(wave_number);
        }
        let results = std::thread::scope(|scope| -> Result<Vec<(bool, Vec<u8>)>, Failure> {
            let mut producers = Vec::new();
            let mut consumers = Vec::new();
            for index in 0..2 {
                let (mut sender, mut receiver) = UnixStream::pair()?;
                sender.set_write_timeout(Some(Duration::from_secs(5)))?;
                receiver.set_read_timeout(Some(Duration::from_secs(5)))?;
                let release = Arc::clone(&gate);
                let arrivals = Arc::clone(&ready);
                let meter = Arc::clone(&meters[index]);
                let source = sources[index];
                let request = requests[index];
                producers.push(scope.spawn(move || -> Result<bool, Failure> {
                    arrivals.fetch_add(1, Ordering::Release);
                    let waiting = Instant::now();
                    while !release.load(Ordering::Acquire) {
                        require(waiting.elapsed() < Duration::from_secs(5), "start-gate deadline")?;
                        std::thread::sleep(Duration::from_millis(1));
                    }
                    let cancelled = AtomicBool::new(false);
                    let now = || start.elapsed().as_nanos().min(u64::MAX as u128) as u64;
                    let mut lease = match meter.reserve(budget::FRAME_BYTES, now, &cancelled) {
                        Ok(lease) => lease,
                        Err(budget::MeterError::CapacityDenied) => return Ok(false),
                        Err(error) => return Err(error.into()),
                    };
                    let mut frame = Vec::new();
                    retrieval::respond_file(&[request], source, &mut frame, &cancelled)?;
                    require(frame.len() == budget::FRAME_BYTES as usize, "frame size")?;
                    let mut sent = 0;
                    while sent < frame.len() {
                        let n = lease.write_to(&mut sender, &frame[sent..], now, &cancelled)?;
                        require(n != 0, "zero-byte write before completion")?;
                        sent += n;
                    }
                    lease.check_completion(now, &cancelled)?;
                    Ok(true)
                }));
                consumers.push(scope.spawn(move || -> Result<Vec<u8>, Failure> {
                    let mut received = Vec::new();
                    (&mut receiver)
                        .take(budget::FRAME_BYTES + 1)
                        .read_to_end(&mut received)?;
                    require(received.len() <= budget::FRAME_BYTES as usize, "extra bytes")?;
                    Ok(received)
                }));
            }
            let waiting = Instant::now();
            while ready.load(Ordering::Acquire) != 2 {
                require(waiting.elapsed() < Duration::from_secs(5), "worker-ready deadline")?;
                std::thread::sleep(Duration::from_millis(1));
            }
            gate.store(true, Ordering::Release);
            let mut paired = Vec::new();
            for (producer, consumer) in producers.into_iter().zip(consumers) {
                let emitted = producer.join().map_err(|_| io::Error::other("producer panic"))??;
                let received = consumer.join().map_err(|_| io::Error::other("consumer panic"))??;
                paired.push((emitted, received));
            }
            Ok(paired)
        })?;
        let mut verified = 0;
        let mut denied = 0;
        for (index, (emitted, frame)) in results.into_iter().enumerate() {
            if emitted {
                let result = retrieval::verify_stream(
                    manifest,
                    &[requests[index]],
                    &mut frame.as_slice(),
                    &AtomicBool::new(false),
                )?;
                require(result.verified == 1 && result.unavailable == 0, "response verification")?;
                verified += 1;
            } else {
                require(frame.is_empty(), "denied response emitted data")?;
                denied += 1;
            }
            let filename = format!("{name}-{wave_number}-{index}.frame");
            let mut file = OpenOptions::new().write(true).create_new(true).open(output.join(filename))?;
            file.write_all(&frame)?;
        }
        Ok((verified, denied))
    }

    pub fn run(directory: &Path) -> Result<(), Failure> {
        // Stable, caller-controlled parent paths are required. No overwrite or cleanup.
        let parent = directory.parent().ok_or_else(|| io::Error::other("parent required"))?;
        require(parent.is_dir(), "parent directory must already exist")?;
        let mut builder = fs::DirBuilder::new();
        use std::os::unix::fs::DirBuilderExt;
        builder.mode(0o700).create(directory)?;
        let source_a = directory.join("source-a.bin");
        let source_b = directory.join("source-b.bin");
        fixture::write_source(&source_a)?;
        fixture::write_source(&source_b)?;
        let manifest = fixture::manifest();
        let mut report = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(directory.join("results.tsv"))?;
        writeln!(report, "case\tverified\tdenied")?;
        let mut total = (0, 0);
        for (name, independent, capacity, separate_sources, waves, expected) in [
            ("two_files_two_meters", true, budget::FRAME_BYTES, true, 1u8, (2, 0)),
            ("shared_tight", false, budget::FRAME_BYTES, false, 1, (1, 1)),
            ("shared_room", false, 2 * budget::FRAME_BYTES, false, 1, (2, 0)),
            ("shared_two_meters", true, budget::FRAME_BYTES, false, 1, (2, 0)),
            ("shared_repeat", false, budget::FRAME_BYTES, false, 2, (1, 3)),
        ] {
            let first = budget::Meter::new(capacity, 0, budget::WINDOW_NANOS)?;
            let second = if independent {
                budget::Meter::new(capacity, 0, budget::WINDOW_NANOS)?
            } else {
                Arc::clone(&first)
            };
            let meters = [first, second];
            let sources = [source_a.as_path(), if separate_sources { source_b.as_path() } else { source_a.as_path() }];
            let start = Instant::now();
            let mut result = (0, 0);
            for w in 0..waves {
                let observed = wave(directory, name, w, sources, &meters, &manifest, start)?;
                result.0 += observed.0;
                result.1 += observed.1;
            }
            require(result == expected, "unexpected case outcome")?;
            for meter in &meters {
                let state = meter.snapshot()?;
                require(state.spent + state.reserved <= state.capacity, "quota conservation")?;
                require(state.reserved == 0 && state.uncertain_spent == 0, "unsettled output")?;
            }
            writeln!(report, "{name}\t{}\t{}", result.0, result.1)?;
            total.0 += result.0;
            total.1 += result.1;
        }
        require(total == (8, 4), "aggregate outcomes")?;
        report.flush()?;
        println!("mode=public_deterministic_local_ipc");
        println!("object_id={}", fixture::hex(&manifest.object_id));
        println!("verified={} capacity_denied={}", total.0, total.1);
        println!("payload_bytes={} wire_bytes={}", total.0 * retrieval::CHUNK_BYTES, total.0 * budget::FRAME_BYTES as usize);
        println!("physical_resource_independence=NotEstablished");
        println!("consensus_eligibility=NotEstablished");
        Ok(())
    }
}

fn main() {
    #[cfg(unix)]
    {
        let mut arguments = std::env::args_os().skip(1);
        let Some(directory) = arguments.next() else {
            eprintln!("usage: service_observation /existing-parent/new-output-directory");
            std::process::exit(2);
        };
        if arguments.next().is_some() {
            eprintln!("exactly one new output directory is required");
            std::process::exit(2);
        }
        if let Err(error) = demo::run(std::path::Path::new(&directory)) {
            eprintln!("local observation did not complete: {error}");
            std::process::exit(1);
        }
    }
    #[cfg(not(unix))]
    {
        eprintln!("This example requires UnixStream IPC; no non-Unix runtime claim.");
        std::process::exit(2);
    }
}
