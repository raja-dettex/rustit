#[allow(unused_imports)]
use std::env;
use std::ffi::CStr;
#[allow(unused_imports)]
use std::fs;
use std::io::BufRead;
use std::io::BufReader;
use std::io::Read;
use anyhow::Context;
use anyhow::Ok;
use clap::{Parser,Subcommand};
use flate2::bufread::ZlibDecoder;
use std::io::Write;

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Command
}
#[derive(Subcommand, Debug)]
pub enum Command { 
    Init,
    CatFile {
        #[clap(short = 'p')]
        pretty_print: bool,
        object_hash: String
    }  
}

enum Kind { 
    Blob
}

fn main() -> anyhow::Result<()>{
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    eprintln!("Logs from your program will appear here!");

    let args = Args::parse();
    match args.command { 
        Command::Init => { 
            fs::create_dir(".git").unwrap();
            fs::create_dir(".git/objects").unwrap();
            fs::create_dir(".git/refs").unwrap();
            fs::write(".git/HEAD", "ref: refs/heads/main\n").unwrap();
            println!("Initialized git directory")
        },
        Command::CatFile { pretty_print, object_hash } => { 
            anyhow::ensure!(pretty_print, "p flag must be there");
            let file = std::fs::File::open(format!(".git/objects/{}/{}", &object_hash[..2], &object_hash[2..])).context("read the hash file")?;
            let reader = BufReader::new(file);
            let mut z = ZlibDecoder::new(reader);
            let mut decoder_reader = BufReader::new(z);
            let mut buf = Vec::new();
            decoder_reader.read_until(0, &mut buf).context("read from .git/objects")?;
            let header = CStr::from_bytes_until_nul(&buf).expect("there is exactly one null bytes");
            let header = header.to_str().context("not valid utf-8 header")?;
            let Some((kind , size)) = header.split_once(' ') else {
                anyhow::bail!("")
            };
            let kind = match kind { 
                "blob" => {
                    Kind::Blob
                },
                _ =>  { 
                    anyhow::bail!("dont know how to print other {kind:?}");
                }
            };


            let size = size.parse::<usize>().context("not valid size")?;
            // buf.clear();
            // buf.resize(size, 0);
            // decoder_reader.read_exact(&mut buf[..]).context("Read exact all the bytes")?;
            // let n = decoder_reader.read(&mut [0 as u8]).context("not end of file")?;
            // anyhow::ensure!(n == 0, "not exact bytes");
            let stdout = std::io::stdout();
            let mut stdout = stdout.lock();
            let mut z = decoder_reader.take(size as u64);
            
            match kind {
                Kind::Blob => {
                    let n = std::io::copy(&mut z, &mut stdout)?;
                    anyhow::ensure!(n as usize == size, "undefined decompression");
                    //stdout.write(&buf).context("write to stdout")?;
                }
            }
            
        }
    }
    // Uncomment this block to pass the first stage
    // let args: Vec<String> = env::args().collect();
    // if args[1] == "init" {
    //     
    // } else {
    //     println!("unknown command: {}", args[1])
    // }
    Ok(())
}


struct LimitReader<R> { 
    reader : R,
    limit: usize
}

impl<R> Read for LimitReader<R>
where R : Read {
    fn read(&mut self, mut buf: &mut [u8]) -> std::io::Result<usize> {
        if buf.len() > self.limit { 
            buf = &mut buf[..self.limit + 1];
        }
        let n = self.reader.read(buf)?;
        if n > self.limit { 
            return  Err(std::io::Error::new(std::io::ErrorKind::InvalidInput, format!("{} trailing bytes", n - self.limit)));
        }
        self.limit -= n;
        std::io::Result::Ok(n)
    }
}